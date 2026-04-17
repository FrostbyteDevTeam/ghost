use ghost_core::uia::element::UiaElement;

pub struct GhostElement {
    inner: UiaElement,
}

impl GhostElement {
    pub fn new(el: UiaElement) -> Self {
        Self { inner: el }
    }

    pub fn name(&self) -> String {
        self.inner.name()
    }
}
