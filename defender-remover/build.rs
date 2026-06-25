// build.rs — Embed .ico icon + UAC elevation manifest into the Windows EXE

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();

        // ── App Icon ──────────────────────────────────────────────────────────
        // Path is relative to the project root (where Cargo.toml lives)
        let icon_path = "../app_icon.ico";
        if std::path::Path::new(icon_path).exists() {
            res.set_icon(icon_path);
        } else {
            // Fallback: check same directory
            let alt = "app_icon.ico";
            if std::path::Path::new(alt).exists() {
                res.set_icon(alt);
            }
        }

        // ── App Metadata ──────────────────────────────────────────────────────
        res.set("FileDescription",  "EdgeDefender Cleaner — Windows Defender + Edge Removal Tool");
        res.set("ProductName",      "EdgeDefender Cleaner");
        res.set("FileVersion",      "13.0.0.0");
        res.set("ProductVersion",   "13.0.0.0");
        res.set("CompanyName",      "CRTY");
        res.set("LegalCopyright",   "CRTY | ionuttbara/windows-defender-remover | ShadowWhisperer/Remove-MS-Edge");
        res.set("OriginalFilename", "EdgeDefenderCleaner.exe");

        // ── UAC Manifest — request Administrator elevation ────────────────────
        // Inline manifest XML (avoids needing a separate file)
        res.set_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="1.0.0.0" processorArchitecture="amd64"
    name="EdgeDefenderCleaner" type="win32"/>
  <description>EdgeDefender Cleaner by CRTY</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 / 11 -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
      <!-- Windows 8.1 -->
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/PM</dpiAware>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>
"#);

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=../app_icon.ico");

        if let Err(e) = res.compile() {
            // Non-fatal: icon embedding failed (e.g. missing windres), continue build
            eprintln!("cargo:warning=winres compile failed: {e}");
        }
    }
}
