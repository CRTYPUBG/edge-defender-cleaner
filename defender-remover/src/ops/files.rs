// src/ops/files.rs — Remove Windows Defender physical files and directories
//
// Mirrors files_removal.bat logic:
//   1. Take ownership of Defender directories
//   2. Grant full permissions to Administrators
//   3. Recursively delete the directories
//
// Paths targeted (from original files_removal.bat):
//   - C:\ProgramData\Microsoft\Windows Defender
//   - C:\Program Files\Windows Defender
//   - C:\Program Files (x86)\Windows Defender
//   - C:\Program Files\Windows Defender Advanced Threat Protection

use anyhow::Result;

/// All Defender directories to be removed
const DEFENDER_DIRS: &[&str] = &[
    r"C:\ProgramData\Microsoft\Windows Defender",
    r"C:\Program Files\Windows Defender",
    r"C:\Program Files (x86)\Windows Defender",
    r"C:\Program Files\Windows Defender Advanced Threat Protection",
];

/// Remove all Defender physical directories.
/// Returns the number of successfully removed locations.
pub fn remove_defender_files() -> Result<usize> {
    let mut count = 0usize;

    for dir in DEFENDER_DIRS {
        let path = std::path::Path::new(dir);

        if !path.exists() {
            crate::utils::logger::log_warn(&format!("Zaten mevcut değil, atlanıyor: {}", dir));
            continue;
        }

        // Step 1: Take ownership via takeown.exe
        let takeown_result = std::process::Command::new("takeown")
            .args(["/f", dir, "/r", "/d", "y"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if takeown_result.is_err() {
            crate::utils::logger::log_warn(&format!("takeown başarısız: {}", dir));
        }

        // Step 2: Grant Administrators full control via icacls
        let icacls_result = std::process::Command::new("icacls")
            .args([dir, "/grant", "administrators:F", "/t"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if icacls_result.is_err() {
            crate::utils::logger::log_warn(&format!("icacls başarısız: {}", dir));
        }

        // Step 3: Remove directory recursively
        match std::fs::remove_dir_all(path) {
            Ok(_) => {
                crate::utils::logger::log_ok(&format!("Silindi: {}", dir));
                count += 1;
            }
            Err(e) => {
                // Try cmd rd as fallback
                let rd_result = std::process::Command::new("cmd")
                    .args(["/c", "rd", "/s", "/q", dir])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();

                match rd_result {
                    Ok(s) if s.success() => {
                        crate::utils::logger::log_ok(&format!("Silindi (fallback): {}", dir));
                        count += 1;
                    }
                    _ => {
                        crate::utils::logger::log_err(dir, &e.to_string());
                    }
                }
            }
        }
    }

    Ok(count)
}
