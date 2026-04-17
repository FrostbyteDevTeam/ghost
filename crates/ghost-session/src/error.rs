#[derive(Debug, thiserror::Error)]
pub enum GhostError {
    #[error("Element not found: {query}")]
    ElementNotFound { query: String, screenshot: Option<Vec<u8>> },

    #[error("Element not interactable: {element} - {reason}")]
    ElementNotInteractable { element: String, reason: String },

    #[error("Ghost stopped by emergency stop (Ctrl+Alt+G)")]
    Stopped,

    #[error("Timeout after {ms}ms waiting for: {action}")]
    Timeout { action: String, ms: u64 },

    #[error("UIA unavailable for app: {app}")]
    UiaUnavailable { app: String },

    #[error("Process not found: {name}")]
    ProcessNotFound { name: String },

    #[error("Core error: {0}")]
    Core(#[from] ghost_core::error::CoreError),
}

pub type Result<T> = std::result::Result<T, GhostError>;
