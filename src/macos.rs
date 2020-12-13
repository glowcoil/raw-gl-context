use raw_window_handle::HasRawWindowHandle;

pub struct GlContext {}

impl GlContext {
    pub fn create(parent: &impl HasRawWindowHandle) -> GlContext {
        GlContext {}
    }
}
