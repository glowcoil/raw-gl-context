use std::ffi::c_void;
use std::str::FromStr;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use cocoa::appkit::{
    NSOpenGLContext, NSOpenGLPFAAccelerated, NSOpenGLPFAAlphaSize, NSOpenGLPFAColorSize,
    NSOpenGLPFADoubleBuffer, NSOpenGLPFAOpenGLProfile, NSOpenGLPixelFormat,
    NSOpenGLProfileVersion3_2Core, NSOpenGLView, NSView,
};
use cocoa::base::{id, nil, YES};
use cocoa::foundation::NSAutoreleasePool;

use core_foundation::base::TCFType;
use core_foundation::bundle::{CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName};
use core_foundation::string::CFString;

use objc::{msg_send, sel, sel_impl};

use crate::GlConfig;

pub struct GlContext {
    view: id,
    context: id,
}

impl GlContext {
    pub fn create(parent: &impl HasRawWindowHandle, config: GlConfig) -> Result<GlContext, ()> {
        let handle = if let RawWindowHandle::MacOS(handle) = parent.raw_window_handle() {
            handle
        } else {
            return Err(());
        };

        if handle.ns_view.is_null() {
            return Err(());
        }

        let parent_view = handle.ns_view as id;

        unsafe {
            #[rustfmt::skip]
            let pixel_format = NSOpenGLPixelFormat::alloc(nil).initWithAttributes_(&[
                NSOpenGLPFAOpenGLProfile as u32, NSOpenGLProfileVersion3_2Core as u32,
                NSOpenGLPFAColorSize as u32, 24u32,
                NSOpenGLPFAAlphaSize as u32, 8u32,
                NSOpenGLPFADoubleBuffer as u32, NSOpenGLPFAAccelerated as u32,
                0u32,
            ]);

            let view = NSOpenGLView::alloc(nil)
                .initWithFrame_pixelFormat_(parent_view.frame(), pixel_format);

            let () = msg_send![view, retain];
            NSOpenGLView::display_(view);
            parent_view.addSubview_(view);

            let context: id = msg_send![view, openGLContext];
            let () = msg_send![context, retain];

            let () = msg_send![pixel_format, release];

            Ok(GlContext { view, context })
        }
    }

    pub fn make_current(&self) {
        unsafe {
            self.context.makeCurrentContext();
        }
    }

    pub fn make_not_current(&self) {
        unsafe {
            NSOpenGLContext::clearCurrentContext(self.context);
        }
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        let symbol_name = CFString::from_str(symbol).unwrap();
        let framework_name = CFString::from_str("com.apple.opengl").unwrap();
        let framework =
            unsafe { CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef()) };
        let addr = unsafe {
            CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef())
        };
        addr as *const c_void
    }

    pub fn swap_buffers(&self) {
        unsafe {
            self.context.flushBuffer();
            let () = msg_send![self.view, setNeedsDisplay: YES];
        }
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            let () = msg_send![self.context, release];
            let () = msg_send![self.view, release];
        }
    }
}
