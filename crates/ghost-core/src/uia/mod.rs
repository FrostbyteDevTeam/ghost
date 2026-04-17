pub mod element;
pub mod patterns;
pub mod tree;

use crate::error::CoreError;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

/// Initialize COM in multithreaded apartment mode.
/// Must be called once per thread before using UIA.
pub fn init_com() -> Result<(), CoreError> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()
            .map_err(|e| CoreError::ComInit(format!("CoInitializeEx failed: {e:?}")))
    }
}
