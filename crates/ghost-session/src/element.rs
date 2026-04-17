use ghost_core::uia::{element::UiaElement, patterns};
use ghost_core::input::hotkey::is_stopped;
use crate::error::{GhostError, Result};

pub struct GhostElement {
    inner: UiaElement,
}

impl GhostElement {
    pub(crate) fn new(inner: UiaElement) -> Self {
        Self { inner }
    }

    /// The element's accessible name.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// The element's bounding rectangle as (left, top, right, bottom).
    pub fn bounding_rect(&self) -> Option<(i32, i32, i32, i32)> {
        self.inner.bounding_rect().map(|r| (r.left, r.top, r.right, r.bottom))
    }

    /// Click this element using InvokePattern or coordinate fallback.
    pub async fn click(&self) -> Result<()> {
        if is_stopped() { return Err(GhostError::Stopped); }
        if !self.inner.is_enabled() {
            return Err(GhostError::ElementNotInteractable {
                element: self.inner.name(),
                reason: "element is disabled".into(),
            });
        }
        patterns::invoke(&self.inner).map_err(GhostError::Core)
    }

    /// Type text into this element using ValuePattern or keyboard fallback.
    pub async fn type_text(&self, text: &str) -> Result<()> {
        if is_stopped() { return Err(GhostError::Stopped); }
        if !self.inner.is_enabled() {
            return Err(GhostError::ElementNotInteractable {
                element: self.inner.name(),
                reason: "element is disabled".into(),
            });
        }
        patterns::set_value(&self.inner, text).map_err(GhostError::Core)
    }
}
