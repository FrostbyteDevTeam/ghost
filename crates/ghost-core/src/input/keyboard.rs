use windows::Win32::UI::Input::KeyboardAndMouse::*;
use crate::error::CoreError;
use super::hotkey::is_stopped;

pub fn key_event(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                ..Default::default()
            },
        },
    }
}

/// Convert text to a sequence of Unicode key events (down+up per char).
pub fn text_to_inputs(text: &str) -> Vec<INPUT> {
    let mut inputs = Vec::new();
    for ch in text.chars() {
        let down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wScan: ch as u16,
                    dwFlags: KEYEVENTF_UNICODE,
                    ..Default::default()
                },
            },
        };
        let up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wScan: ch as u16,
                    dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                    ..Default::default()
                },
            },
        };
        inputs.push(down);
        inputs.push(up);
    }
    inputs
}

/// Type a string into the focused application using Unicode input events.
/// Checks STOP_FLAG between characters.
pub fn type_text(text: &str) -> Result<(), CoreError> {
    for ch in text.chars() {
        if is_stopped() {
            return Err(CoreError::Win32 { code: 0, context: "stopped" });
        }
        let inputs = text_to_inputs(&ch.to_string());
        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                tracing::warn!("type_text: sent {}/{} inputs for char '{}'", sent, inputs.len(), ch);
            }
        }
    }
    Ok(())
}

/// Press and release a virtual key (non-Unicode, for special keys like Enter, Tab, etc.)
pub fn press_key(vk: VIRTUAL_KEY) -> Result<(), CoreError> {
    if is_stopped() {
        return Err(CoreError::Win32 { code: 0, context: "stopped" });
    }
    let inputs = [key_event(vk, false), key_event(vk, true)];
    unsafe {
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            tracing::warn!("press_key: sent {}/{} inputs", sent, inputs.len());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_down_event_has_correct_vk() {
        let input = key_event(VK_A, false);
        unsafe {
            assert_eq!(input.Anonymous.ki.wVk, VK_A);
            assert_eq!(input.Anonymous.ki.dwFlags, KEYBD_EVENT_FLAGS(0));
        }
    }

    #[test]
    fn key_up_event_has_keyup_flag() {
        let input = key_event(VK_A, true);
        unsafe {
            assert_eq!(input.Anonymous.ki.dwFlags, KEYEVENTF_KEYUP);
        }
    }

    #[test]
    fn text_to_inputs_produces_pairs() {
        // "ab" = down(A) + up(A) + down(B) + up(B) = 4 inputs
        let inputs = text_to_inputs("ab");
        assert_eq!(inputs.len(), 4);
    }

    #[test]
    fn empty_text_produces_no_inputs() {
        assert_eq!(text_to_inputs("").len(), 0);
    }
}
