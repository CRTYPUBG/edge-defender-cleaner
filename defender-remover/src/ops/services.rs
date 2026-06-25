// src/ops/services.rs — Windows service control and scheduled task management
//
// Stops and disables Windows Defender services:
//   - WinDefend         (Windows Defender Antivirus Service)
//   - WdNisSvc          (Windows Defender Network Inspection)
//   - WdNisDrv          (Network Inspection System Driver)
//   - SecurityHealthService
//   - SgrmBroker        (System Guard Runtime Monitor)
//   - wscsvc            (Windows Security Center)
//   - Sense             (Windows Defender Advanced Threat Protection)
//   - MpsSvc            (Windows Firewall — optional, disabled on request)
//
// Also disables Defender scheduled tasks via schtasks.

use anyhow::Result;

/// List of Defender-related services to stop + disable
const DEFENDER_SERVICES: &[&str] = &[
    "WinDefend",
    "WdNisSvc",
    "SecurityHealthService",
    "SgrmBroker",
    "wscsvc",
    "Sense",
    "MsSecFlt",
    "WdFilter",
    "WdBoot",
];

/// List of Defender scheduled tasks to disable
const DEFENDER_TASKS: &[&str] = &[
    r"Microsoft\Windows\Windows Defender\Windows Defender Cache Maintenance",
    r"Microsoft\Windows\Windows Defender\Windows Defender Cleanup",
    r"Microsoft\Windows\Windows Defender\Windows Defender Scheduled Scan",
    r"Microsoft\Windows\Windows Defender\Windows Defender Verification",
];

/// Stop and disable all Defender services
pub fn stop_defender_services() -> Result<()> {
    for svc_name in DEFENDER_SERVICES {
        // First try to stop it
        let _ = run_sc("stop", svc_name);
        // Then set start type to DISABLED
        let _ = run_sc_config(svc_name, "disabled");

        crate::utils::logger::log_ok(&format!("Servis durduruldu/devre dışı: {}", svc_name));
    }

    // Also use reg to set ImagePath-based services to 4 (disabled)
    disable_services_via_registry()?;

    Ok(())
}

/// Disable Defender scheduled tasks via schtasks.exe
pub fn disable_defender_tasks() -> Result<()> {
    for task in DEFENDER_TASKS {
        let status = std::process::Command::new("schtasks")
            .args(["/change", "/tn", task, "/disable"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() => {
                crate::utils::logger::log_ok(&format!("Görev devre dışı: {}", task));
            }
            _ => {
                crate::utils::logger::log_warn(&format!("Görev devre dışı bırakılamadı: {}", task));
            }
        }
    }

    Ok(())
}

/// Run sc.exe with given command and service name
fn run_sc(command: &str, service: &str) -> Result<()> {
    std::process::Command::new("sc")
        .args([command, service])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    Ok(())
}

/// Run sc.exe config to change start type
fn run_sc_config(service: &str, start_type: &str) -> Result<()> {
    std::process::Command::new("sc")
        .args(["config", service, &format!("start= {}", start_type)])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    Ok(())
}

/// Disable services via registry (Start = 4)
fn disable_services_via_registry() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let services_root = r"SYSTEM\CurrentControlSet\Services";

    for svc in DEFENDER_SERVICES {
        let svc_path = format!("{}\\{}", services_root, svc);
        if let Ok((key, _)) = hklm.create_subkey(&svc_path) {
            // Start = 4 means "disabled"
            let _ = key.set_value("Start", &4u32);
        }
    }

    Ok(())
}
