// src/ops/edge.rs — Microsoft Edge Removal (WebView2 KORUNUR)
//
// Yapımcı: CRTY
// Kaynak: ShadowWhisperer/Remove-MS-Edge (edge.py portu)
//
// İşlemler:
//  1. Edge kaldır (setup.exe --uninstall) → WebView2 DOKUNULMAZ
//  2. Edge AppX paketlerini registry EndOfLife olarak işaretle
//  3. Masaüstü / Başlat menüsü kısayollarını sil
//  4. Edge zamanlanmış görevlerini sil
//  5. Edge servislerini durdur ve sil
//  6. Edge klasörlerini sil (SystemApps, WindowsApps, Program Files)
//  7. System32'deki Edge .exe dosyalarını sil
//  8. Edge kayıt defteri anahtarlarını temizle
//  9. Edge'in YENIDEN YÜKLENMESINI ENGELLEYEN zamanlanmış görev kur

use anyhow::Result;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

const EDGE_SETUP_X64: &str = r"C:\Program Files (x86)\Microsoft\EdgeUpdate\setup.exe";
const EDGE_SETUP_X86: &str = r"C:\Program Files\Microsoft\EdgeUpdate\setup.exe";
const PROGRAM_FILES_X86: &str = r"C:\Program Files (x86)";
const PROGRAM_FILES: &str = r"C:\Program Files";
const SYSTEM_ROOT: &str = r"C:\Windows";
const PROGRAM_DATA: &str = r"C:\ProgramData";

const EDGE_SERVICES: &[&str] = &[
    "edgeupdate",
    "edgeupdatem",
    "MicrosoftEdgeElevationService",
];

const EDGE_SCHEDULED_TASKS_PREFIX: &str = "MicrosoftEdge";

/// Full registry key paths to delete from HKLM
const EDGE_REG_FULL_PATHS_HKLM: &[&str] = &[
    r"AppID\MicrosoftEdgeUpdate.exe",
    r"SOFTWARE\Classes\microsoft-edge",
    r"SOFTWARE\Classes\microsoft-edge-holographic",
    r"SOFTWARE\Microsoft\Active Setup\Installed Components\{9459C573-B17A-45AE-9F64-1857B5D58CEE}",
    r"SOFTWARE\Microsoft\Edge",
    r"SOFTWARE\Microsoft\EdgeUpdate",
    r"SOFTWARE\Microsoft\Internet Explorer\EdgeDebugActivation",
    r"SOFTWARE\Microsoft\Internet Explorer\EdgeIntegration",
    r"SOFTWARE\Microsoft\MicrosoftEdge",
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\MicrosoftEdgeUpdate.exe",
    r"SOFTWARE\Microsoft\Windows\Shell\Associations\UrlAssociations\microsoft-edge",
    r"SOFTWARE\Microsoft\Windows\Shell\Associations\UrlAssociations\microsoft-edge-holographic",
    r"SOFTWARE\WOW6432Node\Microsoft\Edge",
    r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate",
    r"SYSTEM\CurrentControlSet\Services\edgeupdate",
    r"SYSTEM\CurrentControlSet\Services\edgeupdatem",
    r"SYSTEM\CurrentControlSet\Services\MicrosoftEdgeElevationService",
];

/// Registry paths where we delete subkeys matching "microsoft.microsoftedge*"
const EDGE_WILDCARD_REG_PATHS: &[&str] = &[
    r"ActivatableClasses\Package",
    r"Extensions\ContractId\windows.appExecutionAlias\PackageId",
    r"SOFTWARE\Classes\Extensions\ContractId\Windows.AppService\PackageId",
    r"SOFTWARE\Classes\Extensions\ContractId\Windows.BackgroundTasks\PackageId",
    r"SOFTWARE\Classes\Extensions\ContractId\Windows.Launch\PackageId",
    r"SOFTWARE\Classes\Extensions\ContractId\Windows.Protocol\PackageId",
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\AppHost\IndexedDB",
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\Applications",
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications",
];

/// Remove Microsoft Edge only — WebView2 is PRESERVED
pub fn remove_edge() -> Result<()> {
    // 1. Kill Edge update process
    let _ = std::process::Command::new("taskkill")
        .args(["/IM", "MicrosoftEdgeUpdate.exe", "/F"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // 2. Uninstall Edge via setup.exe
    //    NOTE: WebView2 (--msedgewebview) is intentionally NOT removed.
    //    WebView2 is required by many apps (Xbox, Photos, Roblox, etc.)
    uninstall_edge_only();

    // 3. Mark Edge AppX packages as EndOfLife (NOT WebView packages)
    mark_edge_appx_eol();

    // 4. Remove desktop and start menu shortcuts
    remove_edge_shortcuts();

    // 5. Delete Edge scheduled tasks (Microsoft-installed ones)
    remove_edge_tasks();

    // 6. Delete Edge services
    remove_edge_services();

    // 7. Remove Edge directories (skip EdgeWebView dirs)
    remove_edge_directories();

    // 8. Remove Edge System32 executables
    remove_edge_system32_files();

    // 9. Clean Edge registry keys
    clean_edge_registry();

    // 10. Install anti-reinstall scheduled task guard
    install_edge_block_task();

    crate::utils::logger::log_ok("Microsoft Edge kaldırıldı (WebView2 korundu)");
    Ok(())
}

/// Uninstall Edge only via setup.exe — WebView2 is deliberately skipped.
///
/// WebView2 runtime is kept because it is a dependency of many Windows
/// apps (Xbox App, Windows Mail, Roblox, ImageGlass, etc.).  
/// Only the Edge browser itself is removed.
fn uninstall_edge_only() {
    let setup_paths = [EDGE_SETUP_X64, EDGE_SETUP_X86];

    let edge_dirs = [
        format!("{}\\Microsoft\\Edge\\Application", PROGRAM_FILES_X86),
        format!("{}\\Microsoft\\Edge\\Application", PROGRAM_FILES),
        format!("{}\\Microsoft\\EdgeUpdate", PROGRAM_FILES_X86),
        format!("{}\\Microsoft\\EdgeUpdate", PROGRAM_FILES),
    ];

    let mut setup_exe: Option<PathBuf> = None;

    for p in &setup_paths {
        if Path::new(p).exists() {
            setup_exe = Some(PathBuf::from(p));
            break;
        }
    }

    if setup_exe.is_none() {
        for dir in &edge_dirs {
            let candidate = Path::new(dir).join("setup.exe");
            if candidate.exists() {
                setup_exe = Some(candidate);
                break;
            }
        }
    }

    if let Some(exe) = setup_exe {
        // --uninstall --system-level --force-uninstall
        // NOTE: No --msedgewebview flag → WebView2 untouched
        let _ = std::process::Command::new(&exe)
            .args(["--uninstall", "--system-level", "--force-uninstall"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

/// Legacy function kept for reference — NOT called in normal flow
#[allow(dead_code)]
fn uninstall_via_setup(webview: bool) {
    let setup_paths = [EDGE_SETUP_X64, EDGE_SETUP_X86];

    // Also check in Edge/EdgeUpdate folders
    let edge_dirs = [
        format!("{}\\Microsoft\\Edge\\Application", PROGRAM_FILES_X86),
        format!("{}\\Microsoft\\Edge\\Application", PROGRAM_FILES),
        format!("{}\\Microsoft\\EdgeUpdate", PROGRAM_FILES_X86),
        format!("{}\\Microsoft\\EdgeUpdate", PROGRAM_FILES),
    ];

    let mut setup_exe: Option<PathBuf> = None;

    // Try direct paths
    for p in &setup_paths {
        if Path::new(p).exists() {
            setup_exe = Some(PathBuf::from(p));
            break;
        }
    }

    // If not found directly, search in edge dirs for setup.exe
    if setup_exe.is_none() {
        for dir in &edge_dirs {
            let candidate = Path::new(dir).join("setup.exe");
            if candidate.exists() {
                setup_exe = Some(candidate);
                break;
            }
        }
    }

    if let Some(exe) = setup_exe {
        let mut args = vec![
            "--uninstall".to_string(),
            "--system-level".to_string(),
            "--force-uninstall".to_string(),
        ];
        if webview {
            args.insert(1, "--msedgewebview".to_string());
        }

        let _ = std::process::Command::new(&exe)
            .args(&args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        // Wait briefly for uninstall to proceed
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

/// Mark Edge AppX packages as EndOfLife in registry so Windows removes them
fn mark_edge_appx_eol() {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let store_base = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore";

    // Get list of Edge AppX packages via PowerShell
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "Get-AppxPackage -AllUsers | Where-Object {$_.PackageFullName -ilike '*MicrosoftEdge*'} | Select-Object -ExpandProperty PackageFullName",
        ])
        .output();

    let packages: Vec<String> = match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect(),
        Err(_) => vec![],
    };

    // Get current user SID
    let user_sid = get_user_sid().unwrap_or_else(|| "S-1-5-18".to_string());

    let system_sid = "S-1-5-18";

    for pkg in &packages {
        for sid in &[user_sid.as_str(), system_sid] {
            let path = format!("{}\\EndOfLife\\{}\\{}", store_base, sid, pkg);
            let _ = hklm.create_subkey(&path);
        }
        let deprovisioned_path = format!("{}\\Deprovisioned\\{}", store_base, pkg);
        let _ = hklm.create_subkey(&deprovisioned_path);
    }
}

/// Get current user's SID string
fn get_user_sid() -> Option<String> {
    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-Command",
            "(New-Object System.Security.Principal.NTAccount($env:USERNAME)).Translate([System.Security.Principal.SecurityIdentifier]).Value",
        ])
        .output()
        .ok()?;

    let sid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sid.starts_with("S-1-") {
        Some(sid)
    } else {
        None
    }
}

/// Remove Edge desktop and Start Menu shortcuts
fn remove_edge_shortcuts() {
    // Public Start Menu
    let start_menu = PathBuf::from(PROGRAM_DATA)
        .join(r"Microsoft\Windows\Start Menu\Programs\Microsoft Edge.lnk");
    let _ = std::fs::remove_file(&start_menu);

    // User desktops — get from registry profile list
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList") {
        let profiles: Vec<String> = key.enum_keys().flatten().collect();
        for profile in profiles {
            if let Ok(sub) = key.open_subkey(&profile) {
                if let Ok(path) = sub.get_value::<String, _>("ProfileImagePath") {
                    for link_name in &["edge.lnk", "Microsoft Edge.lnk"] {
                        let link = PathBuf::from(&path).join("Desktop").join(link_name);
                        let _ = std::fs::remove_file(&link);
                    }
                }
            }
        }
    }
}

/// Remove Edge scheduled tasks
fn remove_edge_tasks() {
    // List all tasks and delete ones containing MicrosoftEdge
    let output = std::process::Command::new("schtasks")
        .args(["/query", "/fo", "csv"])
        .output();

    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        let tasks: Vec<String> = stdout
            .lines()
            .skip(1)
            .filter(|line| line.contains(EDGE_SCHEDULED_TASKS_PREFIX))
            .filter_map(|line| {
                // CSV first column is the task name (quoted)
                let name = line.split(',').next()?.trim().trim_matches('"').to_string();
                Some(name)
            })
            .collect();

        for task in tasks {
            let _ = std::process::Command::new("schtasks")
                .args(["/delete", "/tn", &task, "/f"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }

    // Also remove task files from System32\Tasks
    let tasks_path = PathBuf::from(SYSTEM_ROOT).join("System32\\Tasks");
    if tasks_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&tasks_path) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.starts_with("MicrosoftEdge") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
}

/// Delete Edge Windows services
fn remove_edge_services() {
    for svc in EDGE_SERVICES {
        let _ = std::process::Command::new("sc")
            .args(["stop", svc])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = std::process::Command::new("sc")
            .args(["delete", svc])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

/// Remove Edge folders from SystemApps, WindowsApps, Program Files
fn remove_edge_directories() {
    // SystemApps and WindowsApps — look for Microsoft.MicrosoftEdge* folders
    for root in &[
        format!("{}\\SystemApps", SYSTEM_ROOT),
        format!("{}\\WindowsApps", PROGRAM_FILES),
    ] {
        let root_path = Path::new(root);
        if root_path.exists() {
            if let Ok(entries) = std::fs::read_dir(root_path) {
                for entry in entries.flatten() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.starts_with("Microsoft.MicrosoftEdge")
                        || fname.starts_with("Microsoft.MicrosoftEdgeDevToolsClient")
                    {
                        force_remove_dir(&entry.path());
                    }
                }
            }
        }
    }

    // Program Files Edge folders
    // NOTE: "EdgeWebView" is intentionally EXCLUDED to preserve WebView2 runtime
    for base in &[PROGRAM_FILES, PROGRAM_FILES_X86] {
        for folder in &["Edge", "EdgeCore", "EdgeUpdate"] {
            // Skip EdgeWebView — WebView2 must be preserved
            let dir = PathBuf::from(base).join("Microsoft").join(folder);
            if dir.exists() {
                force_remove_dir(&dir);
            }
        }
    }
}

/// Remove MicrosoftEdge*.exe from System32
fn remove_edge_system32_files() {
    let system32 = PathBuf::from(SYSTEM_ROOT).join("System32");
    if let Ok(entries) = std::fs::read_dir(&system32) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname.starts_with("MicrosoftEdge") && fname.ends_with(".exe") {
                let path = entry.path();
                // Take ownership first
                let _ = std::process::Command::new("takeown")
                    .args(["/f", path.to_str().unwrap_or("")])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                let user = std::env::var("USERNAME").unwrap_or_else(|_| "Administrators".to_string());
                let icacls_arg = format!("{}:(OI)(CI)F", user);
                let _ = std::process::Command::new("icacls")
                    .args([
                        path.to_str().unwrap_or(""),
                        "/inheritance:e",
                        "/grant",
                        &icacls_arg,
                        "/T",
                        "/C",
                    ])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

/// Clean all Edge-related registry keys
fn clean_edge_registry() {
    // Full path deletions from HKLM, HKCU, HKCR
    for path in EDGE_REG_FULL_PATHS_HKLM {
        for hive_id in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, HKEY_CLASSES_ROOT] {
            crate::ops::registry::delete_key(*hive_id, path);
        }
    }

    // Wildcard prefix deletions — subkeys starting with "Microsoft.MicrosoftEdge"
    for path in EDGE_WILDCARD_REG_PATHS {
        for hive_id in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER, HKEY_CLASSES_ROOT] {
            crate::ops::registry::delete_subkeys_with_prefix(*hive_id, path, "microsoft.microsoftedge");
            crate::ops::registry::delete_subkeys_with_prefix(*hive_id, path, "Microsoft.MicrosoftEdge");
        }
    }

    // HKCU Software\Classes — MicrosoftEdge* and microsoft-edge*
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(classes) = hkcu.open_subkey("Software\\Classes") {
        let names: Vec<String> = classes.enum_keys().flatten().collect();
        for name in names {
            let lower = name.to_lowercase();
            if lower.starts_with("microsoftedge") || lower.starts_with("microsoft-edge") {
                let _ = hkcu.delete_subkey_all(format!("Software\\Classes\\{}", name));
            }
        }
    }
}

/// Force-remove a directory: takeown → icacls → rmdir
fn force_remove_dir(path: &Path) {
    if !path.exists() {
        return;
    }

    let path_str = path.to_str().unwrap_or("");

    let _ = std::process::Command::new("takeown")
        .args(["/f", path_str, "/r", "/d", "y"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let _ = std::process::Command::new("icacls")
        .args([path_str, "/grant", "BUILTIN\\Administrators:(F)", "/T"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    let _ = std::process::Command::new("icacls")
        .args([path_str, "/grant", "Everyone:(F)", "/T"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Try Rust's fs::remove_dir_all first
    if std::fs::remove_dir_all(path).is_err() {
        // Fallback: cmd rd /s /q
        let _ = std::process::Command::new("cmd")
            .args(["/c", "rd", "/s", "/q", path_str])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

// ─── Anti-Reinstall Scheduled Task ────────────────────────────────────────────

/// Create a Windows Scheduled Task that blocks Edge from being silently
/// reinstalled by Windows Update or by MicrosoftEdgeUpdate.exe.
///
/// Strategy:
///   - Task name: "CRTY_BlockEdgeReinstall"
///   - Trigger: Every 6 hours (catches update windows)
///   - Action: PowerShell script that:
///       1. Kills MicrosoftEdgeUpdate.exe if running
///       2. Deletes EdgeUpdate service if it reappears
///       3. Marks edge AppX packages as EndOfLife again
///       4. Removes any newly created Edge shortcuts
fn install_edge_block_task() {
    const TASK_NAME: &str = "CRTY_BlockEdgeReinstall";

    // PowerShell inline script that the scheduled task will run
    let ps_script = r#"# EdgeDefender Cleaner — Anti-Reinstall Guard
# Yapimci: CRTY | Kaynak: ShadowWhisperer/Remove-MS-Edge
$ErrorActionPreference = 'SilentlyContinue'

# 1. Kill any EdgeUpdate process trying to reinstall Edge
Get-Process -Name 'MicrosoftEdgeUpdate' -ErrorAction SilentlyContinue | Stop-Process -Force

# 2. Delete the EdgeUpdate service if it reappeared
$svcs = @('edgeupdate', 'edgeupdatem', 'MicrosoftEdgeElevationService')
foreach ($svc in $svcs) {
    $s = Get-Service -Name $svc -ErrorAction SilentlyContinue
    if ($s) { sc.exe stop $svc | Out-Null; sc.exe delete $svc | Out-Null }
}

# 3. Re-mark Edge AppX as EndOfLife so it doesn't reinstall
$store = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore'
$pkgs  = Get-AppxPackage -AllUsers -ErrorAction SilentlyContinue |
         Where-Object { $_.PackageFullName -ilike '*MicrosoftEdge*' -and
                        $_.PackageFullName -inotlike '*WebView*' }
foreach ($pkg in $pkgs) {
    $name = $pkg.PackageFullName
    New-Item "$store\EndOfLife\S-1-5-18\$name" -Force | Out-Null
    New-Item "$store\Deprovisioned\$name"      -Force | Out-Null
}

# 4. Remove any re-created Edge shortcuts
$shortcuts = @(
    "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\Microsoft Edge.lnk",
    "$env:PUBLIC\Desktop\Microsoft Edge.lnk"
)
foreach ($s in $shortcuts) { if (Test-Path $s) { Remove-Item $s -Force } }

# 5. Prevent Edge from being set as default browser silently
$regPath = 'HKLM:\SOFTWARE\Policies\Microsoft\Edge'
if (-not (Test-Path $regPath)) { New-Item $regPath -Force | Out-Null }
Set-ItemProperty -Path $regPath -Name 'HideFirstRunExperience' -Value 1 -Type DWord -Force
Set-ItemProperty -Path $regPath -Name 'DefaultBrowserSettingEnabled' -Value 0 -Type DWord -Force
"#;

    // Write the guard script to ProgramData so it persists across reboots
    let script_dir = PathBuf::from(PROGRAM_DATA).join("CRTY\\EdgeGuard");
    let _ = std::fs::create_dir_all(&script_dir);
    let script_path = script_dir.join("block_edge_reinstall.ps1");

    if std::fs::write(&script_path, ps_script).is_err() {
        crate::utils::logger::log_warn("EdgeGuard: Script yazılamadı, zamanlanmış görev atlanıyor");
        return;
    }

    // XML task definition — runs every 6 hours as SYSTEM
    let script_path_str = script_path.to_string_lossy();
    let task_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>CRTY EdgeDefender Cleaner — Blocks Microsoft Edge from being silently reinstalled. Source: ShadowWhisperer/Remove-MS-Edge</Description>
    <Author>CRTY</Author>
    <URI>\CRTY\BlockEdgeReinstall</URI>
  </RegistrationInfo>
  <Triggers>
    <TimeTrigger>
      <Repetition>
        <Interval>PT6H</Interval>
        <StopAtDurationEnd>false</StopAtDurationEnd>
      </Repetition>
      <StartBoundary>2024-01-01T00:00:00</StartBoundary>
      <Enabled>true</Enabled>
    </TimeTrigger>
    <BootTrigger>
      <Enabled>true</Enabled>
      <Delay>PT2M</Delay>
    </BootTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>S-1-5-18</UserId>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <WakeToRun>false</WakeToRun>
    <ExecutionTimeLimit>PT5M</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>powershell.exe</Command>
      <Arguments>-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "{script_path_str}"</Arguments>
    </Exec>
  </Actions>
</Task>
"#
    );

    // Write XML to temp file then register with schtasks /create /xml
    let xml_path = script_dir.join("block_edge_reinstall.xml");
    if std::fs::write(&xml_path, task_xml.as_bytes()).is_err() {
        crate::utils::logger::log_warn("EdgeGuard: XML yazılamadı");
        return;
    }

    // Delete existing task first (ignore error if not found)
    let _ = std::process::Command::new("schtasks")
        .args(["/delete", "/tn", TASK_NAME, "/f"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Register the new task
    let status = std::process::Command::new("schtasks")
        .args([
            "/create",
            "/tn", TASK_NAME,
            "/xml", xml_path.to_str().unwrap_or(""),
            "/f",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() => {
            crate::utils::logger::log_ok(
                &format!("EdgeGuard zamanlanmış görevi kuruldu: {}", TASK_NAME)
            );
        }
        _ => {
            crate::utils::logger::log_warn(
                "EdgeGuard: Zamanlanmış görev kurulamadı (admin yetki gerekebilir)"
            );
        }
    }
}
