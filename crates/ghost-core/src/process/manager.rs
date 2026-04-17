use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS, PROCESSENTRY32W,
};
use windows::Win32::System::Threading::{
    CreateProcessW, TerminateProcess, OpenProcess, PROCESS_CREATION_FLAGS, PROCESS_TERMINATE,
    STARTUPINFOW, PROCESS_INFORMATION,
};
use windows::Win32::Foundation::CloseHandle;
use crate::error::CoreError;

/// Find the PID of a running process by executable name (case-insensitive).
/// Returns None if not found.
pub fn find_pid_by_name(name: &str) -> Option<u32> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let null_pos = entry.szExeFile.iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let proc_name = String::from_utf16_lossy(&entry.szExeFile[..null_pos]);
                if proc_name.eq_ignore_ascii_case(name) {
                    let _ = CloseHandle(snapshot);
                    return Some(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        None
    }
}

/// Launch a process by executable name or full path.
/// Returns the PID of the new process.
pub fn launch(exe: &str) -> Result<u32, CoreError> {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;

    unsafe {
        let si = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();
        let mut cmd: Vec<u16> = OsStr::new(exe).encode_wide().chain(std::iter::once(0)).collect();

        CreateProcessW(
            None,
            windows::core::PWSTR(cmd.as_mut_ptr()),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0),
            None,
            None,
            &si,
            &mut pi,
        ).map_err(|e| CoreError::Win32 { code: e.code().0 as u32, context: "CreateProcessW" })?;
        let _ = CloseHandle(pi.hThread);
        let _ = CloseHandle(pi.hProcess);
        Ok(pi.dwProcessId)
    }
}

/// Kill a process by PID immediately.
pub fn kill(pid: u32) -> Result<(), CoreError> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
            .map_err(|e| CoreError::Win32 { code: e.code().0 as u32, context: "OpenProcess" })?;
        TerminateProcess(handle, 1)
            .map_err(|e| CoreError::Win32 { code: e.code().0 as u32, context: "TerminateProcess" })?;
        let _ = CloseHandle(handle);
        Ok(())
    }
}
