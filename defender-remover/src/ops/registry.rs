// src/ops/registry.rs — Parse and apply .reg files using the winreg crate
//
// Supports:
//   HKLM, HKCR, HKCU, HKU, HKCC
//   DWORD, QWORD, SZ, BINARY, EXPAND_SZ, MULTI_SZ
//   Key deletion [-HKEY...] and value deletion (value=-)

use anyhow::{Context, Result};
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

/// Apply all .reg files found in a given prefix inside a ZIP archive (e.g. res.crty)
pub fn apply_reg_archive(archive_path: &Path, prefix: &str) -> Result<usize> {
    if !archive_path.exists() {
        anyhow::bail!("Arşiv bulunamadı: {}", archive_path.display());
    }

    let encrypted_payload = std::fs::read(archive_path)?;
    let decrypted_payload = crate::crypto::decrypt_and_verify(&encrypted_payload)?;

    let cursor = std::io::Cursor::new(decrypted_payload);
    let mut archive = zip::ZipArchive::new(cursor)
        .with_context(|| format!("Geçersiz veya bozuk arşiv (deşifre sonrası): {}", archive_path.display()))?;
    
    let mut count = 0usize;
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let name = file.name().to_string();
        
        // Ex: prefix = "Remove_Defender", name = "Remove_Defender/A.reg"
        if name.starts_with(prefix) && name.ends_with(".reg") {
            let mut raw = Vec::new();
            use std::io::Read;
            if file.read_to_end(&mut raw).is_ok() {
                let content = decode_reg_content(&raw);
                match apply_reg_memory(&content) {
                    Ok(_) => {
                        crate::utils::logger::log_ok(&format!("REG uygulandı: {}", name));
                        count += 1;
                    }
                    Err(e) => {
                        crate::utils::logger::log_err(&name, &e.to_string());
                    }
                }
            }
        }
    }
    Ok(count)
}

/// Apply all .reg files found in the given directory. Returns file count.
pub fn apply_reg_directory(dir: &Path) -> Result<usize> {
    if !dir.exists() {
        anyhow::bail!("Dizin bulunamadı: {}", dir.display());
    }

    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Dizin okunamadı: {}", dir.display()))?;

    let mut count = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("reg") {
            match apply_reg_file(&path) {
                Ok(_) => {
                    crate::utils::logger::log_ok(&format!(
                        "REG uygulandı: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ));
                    count += 1;
                }
                Err(e) => {
                    crate::utils::logger::log_err(
                        &path.file_name().unwrap_or_default().to_string_lossy(),
                        &e.to_string(),
                    );
                }
            }
        }
    }
    Ok(count)
}

/// Parse and apply a single .reg file
pub fn apply_reg_file(path: &Path) -> Result<()> {
    let raw = std::fs::read(path)
        .with_context(|| format!("Dosya okunamadı: {}", path.display()))?;
    let content = decode_reg_content(&raw);
    apply_reg_memory(&content)
}

/// Parse and apply registry contents from a string buffer
pub fn apply_reg_memory(content: &str) -> Result<()> {
    let mut current_key_path: Option<String> = None;
    let mut line_buf = String::new();

    for raw_line in content.lines() {
        let trimmed = raw_line.trim_end();

        // Handle line continuation (backslash at end)
        if trimmed.ends_with('\\') {
            line_buf.push_str(trimmed.trim_end_matches('\\'));
            continue;
        } else {
            line_buf.push_str(trimmed);
        }

        let line = line_buf.trim().to_string();
        line_buf.clear();

        if line.is_empty() || line.starts_with(';') || line.starts_with("Windows Registry Editor") {
            continue;
        }

        // Key deletion: [-HKEY_...
        if line.starts_with("[-") && line.ends_with(']') {
            let key_str = &line[2..line.len() - 1];
            delete_key_by_path(key_str);
            current_key_path = None;
            continue;
        }

        // Key definition: [HKEY_...
        if line.starts_with('[') && line.ends_with(']') {
            current_key_path = Some(line[1..line.len() - 1].to_string());
            continue;
        }

        // Value line
        if let Some(ref key_path) = current_key_path.clone() {
            let _ = apply_value(key_path, &line);
        }
    }
    Ok(())
}

/// Delete a registry key (and all subkeys) by full path string
fn delete_key_by_path(full_path: &str) {
    if let Some((hive, sub)) = split_hive(full_path) {
        let _ = hive.delete_subkey_all(sub);
    }
}

/// Delete a registry key recursively by hive + path
pub fn delete_key(hive_id: isize, path: &str) {
    let hive = RegKey::predef(hive_id);
    let _ = hive.delete_subkey_all(path);
}

/// Delete all subkeys matching a prefix pattern under a given hive + path
pub fn delete_subkeys_with_prefix(hive_id: isize, path: &str, prefix: &str) {
    let hive = RegKey::predef(hive_id);
    if let Ok(key) = hive.open_subkey(path) {
        // Collect names first to avoid borrow issues
        let names: Vec<String> = key.enum_keys().flatten().collect();
        for name in names {
            let lower = name.to_lowercase();
            let prefix_lower = prefix.to_lowercase();
            if lower.starts_with(&prefix_lower) {
                let full_path = format!("{}\\{}", path, name);
                let _ = hive.delete_subkey_all(&full_path);
            }
        }
    }
}

/// Parse a value line and write it to the registry
fn apply_value(key_path: &str, line: &str) -> Result<()> {
    let (hive, sub) = split_hive(key_path)
        .ok_or_else(|| anyhow::anyhow!("Bilinmeyen hive: {}", key_path))?;
    let reg_key = hive.create_subkey(sub)?.0;

    // Parse name
    let (name, rest): (String, &str) = if line.starts_with('@') {
        ("".to_string(), &line[1..])
    } else if line.starts_with('"') {
        let content = &line[1..];
        if let Some(close) = content.find('"') {
            let name = content[..close].to_string();
            let rest = &content[close + 1..];
            (name, rest)
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    let rest = rest.trim_start_matches('=');

    // Value deletion
    if rest == "-" {
        let _ = reg_key.delete_value(&name);
        return Ok(());
    }

    // DWORD  dword:XXXXXXXX
    if let Some(hex) = rest.strip_prefix("dword:") {
        let val = u32::from_str_radix(hex.trim(), 16).unwrap_or(0);
        let _ = reg_key.set_value(&name, &val);
        return Ok(());
    }

    // QWORD  qword:XXXXXXXXXXXXXXXX
    if let Some(hex) = rest.strip_prefix("qword:") {
        let val = u64::from_str_radix(hex.trim(), 16).unwrap_or(0);
        let _ = reg_key.set_value(&name, &val);
        return Ok(());
    }

    // Quoted string  "value"
    if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
        let inner = &rest[1..rest.len() - 1];
        let unescaped = inner.replace("\\\"", "\"").replace("\\\\", "\\");
        let _ = reg_key.set_value(&name, &unescaped);
        return Ok(());
    }

    // hex(4): — REG_DWORD stored as 4 little-endian bytes
    if let Some(hex_str) = rest.strip_prefix("hex(4):") {
        let bytes = parse_hex_bytes(hex_str);
        if bytes.len() >= 4 {
            let val = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let _ = reg_key.set_value(&name, &val);
        }
        return Ok(());
    }

    // hex(b): — REG_QWORD stored as 8 little-endian bytes
    if let Some(hex_str) = rest.strip_prefix("hex(b):") {
        let bytes = parse_hex_bytes(hex_str);
        if bytes.len() >= 8 {
            let val = u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
                bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            let _ = reg_key.set_value(&name, &val);
        }
        return Ok(());
    }

    // hex(2): — REG_EXPAND_SZ stored as UTF-16LE bytes
    if let Some(hex_str) = rest.strip_prefix("hex(2):") {
        let bytes = parse_hex_bytes(hex_str);
        let rv = winreg::RegValue {
            bytes,
            vtype: winreg::enums::RegType::REG_EXPAND_SZ,
        };
        let _ = reg_key.set_raw_value(&name, &rv);
        return Ok(());
    }

    // hex(7): — REG_MULTI_SZ stored as UTF-16LE bytes (double-null terminated)
    if let Some(hex_str) = rest.strip_prefix("hex(7):") {
        let bytes = parse_hex_bytes(hex_str);
        let rv = winreg::RegValue {
            bytes,
            vtype: winreg::enums::RegType::REG_MULTI_SZ,
        };
        let _ = reg_key.set_raw_value(&name, &rv);
        return Ok(());
    }

    // hex: — REG_BINARY (raw bytes)
    if let Some(hex_str) = rest.strip_prefix("hex:") {
        let bytes = parse_hex_bytes(hex_str);
        let rv = winreg::RegValue {
            bytes,
            vtype: winreg::enums::RegType::REG_BINARY,
        };
        let _ = reg_key.set_raw_value(&name, &rv);
        return Ok(());
    }

    Ok(())
}

/// Parse a comma-separated hex string like "00,01,02,ff" → Vec<u8>
fn parse_hex_bytes(hex_str: &str) -> Vec<u8> {
    hex_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| u8::from_str_radix(s, 16).ok())
        .collect()
}

/// Split a full registry path into (RegKey hive handle, subpath &str)
fn split_hive(full_path: &str) -> Option<(RegKey, &str)> {
    let pos = full_path.find('\\')?;
    let hive_name = &full_path[..pos];
    let sub = &full_path[pos + 1..];

    let hive = match hive_name.to_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE"  | "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKEY_CLASSES_ROOT"   | "HKCR" => RegKey::predef(HKEY_CLASSES_ROOT),
        "HKEY_CURRENT_USER"   | "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        "HKEY_USERS"          | "HKU"  => RegKey::predef(HKEY_USERS),
        "HKEY_CURRENT_CONFIG" | "HKCC" => RegKey::predef(HKEY_CURRENT_CONFIG),
        _ => return None,
    };
    Some((hive, sub))
}

/// Decode .reg file bytes: UTF-16LE BOM or UTF-8
fn decode_reg_content(raw: &[u8]) -> String {
    if raw.len() >= 2 && raw[0] == 0xFF && raw[1] == 0xFE {
        let utf16: Vec<u16> = raw[2..]
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)]))
            .collect();
        String::from_utf16_lossy(&utf16).to_string()
    } else {
        String::from_utf8_lossy(raw).into_owned()
    }
}
