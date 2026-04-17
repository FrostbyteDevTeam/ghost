use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::GetLastError;
use crate::error::CoreError;

pub static STOP_FLAG: AtomicBool = AtomicBool::new(false);

pub fn is_stopped() -> bool {
    STOP_FLAG.load(Ordering::SeqCst)
}

pub fn trigger_stop() {
    STOP_FLAG.store(true, Ordering::SeqCst);
}

pub fn reset_stop() {
    STOP_FLAG.store(false, Ordering::SeqCst);
}

/// Register Ctrl+Alt+G as a global hotkey (ID=1).
/// Spawns a background thread that listens for WM_HOTKEY messages.
/// On trigger: sets STOP_FLAG, releases all modifier keys.
pub fn register_emergency_stop() -> Result<(), CoreError> {
    unsafe {
        RegisterHotKey(None, 1, MOD_CONTROL | MOD_ALT, b'G' as u32)
            .map_err(|_| CoreError::Win32 { code: GetLastError().0, context: "RegisterHotKey" })?;
    }

    std::thread::spawn(|| {
        let mut msg = MSG::default();
        unsafe {
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                if msg.message == WM_HOTKEY && msg.wParam.0 == 1 {
                    tracing::warn!("Emergency stop triggered (Ctrl+Alt+G)");
                    trigger_stop();
                    release_all_modifiers();
                }
            }
        }
    });

    Ok(())
}

/// Send key-up events for all modifier keys so no key stays stuck.
pub fn release_all_modifiers() {
    let modifiers = [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN];
    let inputs: Vec<INPUT> = modifiers
        .iter()
        .map(|&vk| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    dwFlags: KEYEVENTF_KEYUP,
                    ..Default::default()
                },
            },
        })
        .collect();

    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_flag_starts_false() {
        STOP_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
        assert!(!is_stopped());
    }

    #[test]
    fn stop_flag_set_and_reset() {
        STOP_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
        trigger_stop();
        assert!(is_stopped());
        reset_stop();
        assert!(!is_stopped());
    }
}
