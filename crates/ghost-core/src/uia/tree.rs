use windows::Win32::UI::Accessibility::*;
use windows::Win32::System::Com::CoCreateInstance;
use windows::Win32::System::Com::CLSCTX_INPROC_SERVER;
use super::element::{UiaElement, role_id_to_name};
use crate::error::CoreError;

pub struct UiaTree {
    automation: IUIAutomation,
}

impl UiaTree {
    pub fn new() -> Result<Self, CoreError> {
        unsafe {
            let automation: IUIAutomation = CoCreateInstance(
                &CUIAutomation8,
                None,
                CLSCTX_INPROC_SERVER,
            ).map_err(|e| CoreError::ComInit(e.to_string()))?;
            Ok(Self { automation })
        }
    }

    /// Find first element whose name contains `name` (case-insensitive).
    pub fn find_by_name(&self, name: &str) -> Result<Option<UiaElement>, CoreError> {
        let name_lower = name.to_lowercase();
        unsafe {
            let root = self.automation.GetRootElement()
                .map_err(|e| CoreError::ComInit(e.to_string()))?;
            self.search_subtree_by_name(&root, &name_lower)
        }
    }

    /// Find first element matching the given role name (e.g. "edit", "button").
    pub fn find_by_role(&self, role: &str) -> Result<Option<UiaElement>, CoreError> {
        unsafe {
            let root = self.automation.GetRootElement()
                .map_err(|e| CoreError::ComInit(e.to_string()))?;
            self.search_subtree_by_role(&root, role)
        }
    }

    unsafe fn get_walker(&self) -> Result<IUIAutomationTreeWalker, CoreError> {
        self.automation.ControlViewWalker()
            .map_err(|e| CoreError::ComInit(e.to_string()))
    }

    unsafe fn search_subtree_by_name(
        &self,
        element: &IUIAutomationElement,
        name: &str,
    ) -> Result<Option<UiaElement>, CoreError> {
        let el = UiaElement(element.clone());
        if el.name().to_lowercase().contains(name) {
            return Ok(Some(el));
        }
        let walker = self.get_walker()?;
        let mut child = walker.GetFirstChildElement(element).ok();
        while let Some(c) = child {
            if let Some(found) = self.search_subtree_by_name(&c, name)? {
                return Ok(Some(found));
            }
            child = walker.GetNextSiblingElement(&c).ok();
        }
        Ok(None)
    }

    unsafe fn search_subtree_by_role(
        &self,
        element: &IUIAutomationElement,
        role: &str,
    ) -> Result<Option<UiaElement>, CoreError> {
        let el = UiaElement(element.clone());
        if role_id_to_name(el.control_type()) == role {
            return Ok(Some(el));
        }
        let walker = self.get_walker()?;
        let mut child = walker.GetFirstChildElement(element).ok();
        while let Some(c) = child {
            if let Some(found) = self.search_subtree_by_role(&c, role)? {
                return Ok(Some(found));
            }
            child = walker.GetNextSiblingElement(&c).ok();
        }
        Ok(None)
    }
}
