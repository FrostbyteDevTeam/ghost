use windows::Win32::UI::Accessibility::*;
use windows::core::Interface;
use super::element::UiaElement;
use crate::error::CoreError;

/// Invoke an element via InvokePattern (buttons, links).
/// Falls back to clicking center of bounding rect if InvokePattern unavailable.
pub fn invoke(element: &UiaElement) -> Result<(), CoreError> {
    unsafe {
        if let Ok(pattern) = element.0.GetCurrentPattern(UIA_InvokePatternId) {
            let invoke: IUIAutomationInvokePattern = pattern.cast()
                .map_err(|e| CoreError::Win32 { code: e.code().0 as u32, context: "InvokePattern cast" })?;
            invoke.Invoke()
                .map_err(|e| CoreError::Win32 { code: e.code().0 as u32, context: "InvokePattern.Invoke" })?;
            return Ok(());
        }
    }
    // Coordinate fallback
    if let Some(rect) = element.bounding_rect() {
        let (cx, cy) = rect.center();
        crate::input::mouse::click(cx, cy)?;
    }
    Ok(())
}

/// Set value via ValuePattern (text inputs).
/// Falls back to clicking + typing if ValuePattern unavailable.
pub fn set_value(element: &UiaElement, value: &str) -> Result<(), CoreError> {
    unsafe {
        if let Ok(pattern) = element.0.GetCurrentPattern(UIA_ValuePatternId) {
            let vp: IUIAutomationValuePattern = pattern.cast()
                .map_err(|e| CoreError::Win32 { code: e.code().0 as u32, context: "ValuePattern cast" })?;
            let bstr = windows::core::BSTR::from(value);
            vp.SetValue(&bstr)
                .map_err(|e| CoreError::Win32 { code: e.code().0 as u32, context: "ValuePattern.SetValue" })?;
            return Ok(());
        }
    }
    // Fallback: click to focus, then type
    if let Some(rect) = element.bounding_rect() {
        let (cx, cy) = rect.center();
        crate::input::mouse::click(cx, cy)?;
    }
    crate::input::keyboard::type_text(value)
}
