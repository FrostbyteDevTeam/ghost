use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use crate::error::CoreError;
use super::hotkey::is_stopped;

/// Convert pixel coordinates to Windows absolute mouse coordinates (0-65535 range).
pub fn to_absolute(x: i32, y: i32) -> (i32, i32) {
    unsafe {
        let sw = GetSystemMetrics(SM_CXSCREEN);
        let sh = GetSystemMetrics(SM_CYSCREEN);
        if sw == 0 || sh == 0 {
            return (0, 0);
        }
        ((x * 65535) / sw, (y * 65535) / sh)
    }
}

pub fn move_event(x: i32, y: i32) -> INPUT {
    let (ax, ay) = to_absolute(x, y);
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: ax,
                dy: ay,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                ..Default::default()
            },
        },
    }
}

pub fn click_event(up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dwFlags: if up { MOUSEEVENTF_LEFTUP } else { MOUSEEVENTF_LEFTDOWN },
                ..Default::default()
            },
        },
    }
}

/// Move mouse to pixel coordinates (x, y) and left-click.
pub fn click(x: i32, y: i32) -> Result<(), CoreError> {
    if is_stopped() {
        return Err(CoreError::Win32 { code: 0, context: "stopped" });
    }
    let inputs = [move_event(x, y), click_event(false), click_event(true)];
    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            tracing::warn!("click: sent {}/{} inputs", sent, inputs.len());
        }
    }
    Ok(())
}

/// Move mouse to pixel coordinates without clicking.
pub fn move_to(x: i32, y: i32) -> Result<(), CoreError> {
    if is_stopped() {
        return Err(CoreError::Win32 { code: 0, context: "stopped" });
    }
    let inputs = [move_event(x, y)];
    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != 1 {
            tracing::warn!("move_to: failed to send mouse move");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_event_uses_absolute_flag() {
        let input = move_event(500, 400);
        unsafe {
            assert!(input.Anonymous.mi.dwFlags.contains(MOUSEEVENTF_MOVE));
            assert!(input.Anonymous.mi.dwFlags.contains(MOUSEEVENTF_ABSOLUTE));
        }
    }

    #[test]
    fn click_down_uses_leftdown_flag() {
        let input = click_event(false);
        unsafe {
            assert!(input.Anonymous.mi.dwFlags.contains(MOUSEEVENTF_LEFTDOWN));
        }
    }

    #[test]
    fn click_up_uses_leftup_flag() {
        let input = click_event(true);
        unsafe {
            assert!(input.Anonymous.mi.dwFlags.contains(MOUSEEVENTF_LEFTUP));
        }
    }

    #[test]
    fn to_absolute_maps_zero_to_zero() {
        // At x=0, absolute coord should be 0
        // (this tests the formula direction, not exact values since screen size varies)
        let (ax, _ay) = to_absolute(0, 0);
        assert_eq!(ax, 0);
    }
}
