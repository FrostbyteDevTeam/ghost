#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Win32 error {code:#010x} in {context}")]
    Win32 { code: u32, context: &'static str },

    #[error("COM initialization failed: {0}")]
    ComInit(String),

    #[error("UIA not available for process: {process}")]
    UiaUnavailable { process: String },

    #[error("Process not found: {name}")]
    ProcessNotFound { name: String },
}
