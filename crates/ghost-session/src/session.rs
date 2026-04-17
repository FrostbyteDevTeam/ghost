use std::time::Duration;
use tokio::time::timeout;
use ghost_core::{
    capture::capture_screen,
    input::hotkey::{register_emergency_stop, is_stopped, reset_stop},
    process::launch as proc_launch,
    uia::{init_com, tree::UiaTree},
};
use crate::{
    locator::By,
    error::{GhostError, Result},
};

pub struct Region;

impl Region {
    pub fn full() -> Self {
        Region
    }
}

pub struct GhostSession {
    timeout_ms: u64,
    tree: UiaTree,
}

impl GhostSession {
    /// Create a new automation session.
    /// Initializes COM, registers the Ctrl+Alt+G emergency stop hotkey, and creates the UIA tree.
    pub fn new() -> Result<Self> {
        init_com().map_err(GhostError::Core)?;
        register_emergency_stop().map_err(GhostError::Core)?;
        let tree = UiaTree::new().map_err(GhostError::Core)?;
        Ok(Self {
            timeout_ms: 5000,
            tree,
        })
    }

    /// Override the per-action timeout (default: 5000ms).
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Find the first element matching the locator, retrying until timeout.
    pub async fn find(&self, by: By) -> Result<crate::GhostElement> {
        if is_stopped() {
            return Err(GhostError::Stopped);
        }
        let action = by.to_string();
        let ms = self.timeout_ms;

        let result = timeout(Duration::from_millis(ms), async {
            loop {
                if is_stopped() {
                    return Err(GhostError::Stopped);
                }
                let found = match &by {
                    By::Name(n) => self.tree.find_by_name(n).map_err(GhostError::Core)?,
                    By::Role(r) => self.tree.find_by_role(r).map_err(GhostError::Core)?,
                };
                if let Some(el) = found {
                    return Ok(crate::GhostElement::new(el));
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;

        match result {
            Ok(r) => r,
            Err(_elapsed) => {
                let screenshot = capture_screen().ok();
                Err(GhostError::ElementNotFound {
                    query: action,
                    screenshot,
                })
            }
        }
    }

    /// Click at absolute pixel coordinates without finding an element.
    pub async fn click_at(&self, x: i32, y: i32) -> Result<()> {
        if is_stopped() {
            return Err(GhostError::Stopped);
        }
        ghost_core::input::mouse::click(x, y).map_err(GhostError::Core)
    }

    /// Capture the primary monitor as PNG bytes.
    pub async fn screenshot(&self, _region: Region) -> Result<Vec<u8>> {
        capture_screen().map_err(GhostError::Core)
    }

    /// Launch a process by name or path. Returns PID.
    pub async fn launch(&self, exe: &str) -> Result<u32> {
        proc_launch(exe).map_err(GhostError::Core)
    }

    /// Trigger emergency stop: halts all automation, releases modifier keys.
    pub fn stop(&self) {
        ghost_core::input::hotkey::trigger_stop();
        ghost_core::input::hotkey::release_all_modifiers();
    }

    /// Reset the stop flag (allows automation to resume after a stop).
    pub fn reset(&self) {
        reset_stop();
    }
}
