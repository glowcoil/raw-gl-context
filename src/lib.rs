use raw_window_handle::HasRawWindowHandle;

use std::ffi::c_void;
use std::marker::PhantomData;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
use win as platform;

#[cfg(target_os = "linux")]
mod x11;
#[cfg(target_os = "linux")]
use crate::x11 as platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

pub struct GlContext {
    context: platform::GlContext,
    phantom: PhantomData<*mut ()>,
}

impl GlContext {
    pub fn create(parent: &impl HasRawWindowHandle) -> Result<GlContext, ()> {
        platform::GlContext::create(parent).map(|context| GlContext {
            context,
            phantom: PhantomData,
        })
    }

    pub fn make_current(&self) {
        self.context.make_current();
    }

    pub fn make_not_current(&self) {
        self.context.make_not_current();
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.context.get_proc_address(symbol)
    }

    pub fn swap_buffers(&self) {
        self.context.swap_buffers();
    }
}
