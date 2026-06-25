// src/utils/admin.rs — Windows administrator privilege detection and UAC re-launch

use anyhow::Result;

/// Returns true if the current process has Administrator privileges
#[cfg(target_os = "windows")]
pub fn is_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length: u32 = 0;
        let size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            size,
            &mut return_length,
        );

        let _ = windows::Win32::Foundation::CloseHandle(token);

        result.is_ok() && elevation.TokenIsElevated != 0
    }
}

/// Fallback for non-Windows builds
#[cfg(not(target_os = "windows"))]
pub fn is_elevated() -> bool {
    false
}

/// Re-launch the current executable with UAC elevation prompt
#[cfg(target_os = "windows")]
pub fn relaunch_as_admin() -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe()?;
    let exe_path: Vec<u16> = OsStr::new(exe.to_str().unwrap_or(""))
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let verb: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(exe_path.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        );
    }

    Ok(())
}

/// Fallback for non-Windows
#[cfg(not(target_os = "windows"))]
pub fn relaunch_as_admin() -> Result<()> {
    eprintln!("Bu araç yalnızca Windows üzerinde çalışır.");
    Ok(())
}
