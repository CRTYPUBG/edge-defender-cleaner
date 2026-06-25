// src/ops/appx.rs — Remove SecHealthUI (Windows Security UWP App)
//
// Mirrors RemoveSecHealthApp.ps1 logic by spawning a PowerShell process.
// The PowerShell script must run with elevated (System-level) permissions
// equivalent to what PowerRun provides, so we call PowerShell directly
// with the full script content inlined.

use anyhow::Result;

/// The full PowerShell script for removing SecHealthUI,
/// adapted from RemoveSecHealthApp.ps1
const REMOVE_SEC_HEALTH_UI_SCRIPT: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
$remove_appx = @("SecHealthUI")
$provisioned = Get-AppxProvisionedPackage -Online
$appxpackage = Get-AppxPackage -AllUsers
$eol = @()
$skip = @()
$store = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore'
$users = @('S-1-5-18')
if (Test-Path $store) {
    $users += @((Get-ChildItem $store -ErrorAction SilentlyContinue | Where-Object { $_ -like '*S-1-5-21*' }).PSChildName)
}
foreach ($choice in $remove_appx) {
    if ('' -eq $choice.Trim()) { continue }
    foreach ($appx in ($provisioned | Where-Object { $_.PackageName -like "*$choice*" })) {
        $next = $false
        foreach ($no in $skip) { if ($appx.PackageName -like "*$no*") { $next = $true } }
        if ($next) { continue }
        $PackageName = $appx.PackageName
        $PackageFamilyName = ($appxpackage | Where-Object { $_.Name -eq $appx.DisplayName }).PackageFamilyName
        New-Item "$store\Deprovisioned\$PackageFamilyName" -Force | Out-Null
        foreach ($sid in $users) { New-Item "$store\EndOfLife\$sid\$PackageName" -Force | Out-Null }
        $eol += $PackageName
        dism /online /set-nonremovableapppolicy /packagefamily:$PackageFamilyName /nonremovable:0 | Out-Null
        Remove-AppxProvisionedPackage -PackageName $PackageName -Online -AllUsers | Out-Null
    }
    foreach ($appx in ($appxpackage | Where-Object { $_.PackageFullName -like "*$choice*" })) {
        $next = $false
        foreach ($no in $skip) { if ($appx.PackageFullName -like "*$no*") { $next = $true } }
        if ($next) { continue }
        $PackageFullName = $appx.PackageFullName
        New-Item "$store\Deprovisioned\$($appx.PackageFamilyName)" -Force | Out-Null
        foreach ($sid in $users) { New-Item "$store\EndOfLife\$sid\$PackageFullName" -Force | Out-Null }
        $eol += $PackageFullName
        dism /online /set-nonremovableapppolicy /packagefamily:$($appx.PackageFamilyName) /nonremovable:0 | Out-Null
        Remove-AppxPackage -Package $PackageFullName -AllUsers | Out-Null
    }
}
Write-Output "SecHealthUI removal complete."
"#;

/// Remove SecHealthUI by spawning an elevated PowerShell process
pub fn remove_sec_health_ui() -> Result<()> {
    // Write script to a temp file in the system temp dir
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("remove_sec_health_ui_temp.ps1");

    std::fs::write(&script_path, REMOVE_SEC_HEALTH_UI_SCRIPT)?;

    let output = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_path.to_str().unwrap_or(""),
        ])
        .output()?;

    // Clean up temp file
    let _ = std::fs::remove_file(&script_path);

    if output.status.success() {
        crate::utils::logger::log_ok("SecHealthUI başarıyla kaldırıldı");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        crate::utils::logger::log_warn(&format!("SecHealthUI kısmen kaldırıldı: {}", stderr));
        // Non-fatal: continue even if this step fails
        Ok(())
    }
}
