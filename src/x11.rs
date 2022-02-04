use std::ffi::{c_void, CString};
use std::os::raw::{c_int, c_ulong};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use x11::glx;
use x11::xlib;

use crate::{GlConfig, GlError, Profile};

mod errors;

#[derive(Debug)]
pub enum CreationFailedError {
    InvalidFBConfig,
    GetProcAddressFailed,
    MakeCurrentFailed,
    ContextCreationFailed,
    X11Error(errors::XLibError)
}

impl From<errors::XLibError> for GlError {
    fn from(e: errors::XLibError) -> Self {
        GlError::CreationFailed(CreationFailedError::X11Error(e))
    }
}

// See https://www.khronos.org/registry/OpenGL/extensions/ARB/GLX_ARB_create_context.txt

type GlXCreateContextAttribsARB = unsafe extern "C" fn(
    dpy: *mut xlib::Display,
    fbc: glx::GLXFBConfig,
    share_context: glx::GLXContext,
    direct: xlib::Bool,
    attribs: *const c_int,
) -> glx::GLXContext;

// See https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_swap_control.txt

type GlXSwapIntervalEXT =
    unsafe extern "C" fn(dpy: *mut xlib::Display, drawable: glx::GLXDrawable, interval: i32);

// See https://www.khronos.org/registry/OpenGL/extensions/ARB/ARB_framebuffer_sRGB.txt

const GLX_FRAMEBUFFER_SRGB_CAPABLE_ARB: i32 = 0x20B2;

fn get_proc_address(symbol: &str) -> *const c_void {
    let symbol = CString::new(symbol).unwrap();
    unsafe { glx::glXGetProcAddress(symbol.as_ptr() as *const u8).unwrap() as *const c_void }
}

pub struct GlContext {
    window: c_ulong,
    display: *mut xlib::_XDisplay,
    context: glx::GLXContext,
}

impl GlContext {
    pub unsafe fn create(
        parent: &impl HasRawWindowHandle,
        config: GlConfig,
    ) -> Result<GlContext, GlError> {
        let handle = if let RawWindowHandle::Xlib(handle) = parent.raw_window_handle() {
            handle
        } else {
            return Err(GlError::InvalidWindowHandle);
        };

        if handle.display.is_null() {
            return Err(GlError::InvalidWindowHandle);
        }

        let display = handle.display as *mut xlib::_XDisplay;

        errors::XErrorHandler::handle(display, |error_handler| {
            let screen = unsafe { xlib::XDefaultScreen(display) };

            #[rustfmt::skip]
                let fb_attribs = [
                glx::GLX_X_RENDERABLE, 1,
                glx::GLX_X_VISUAL_TYPE, glx::GLX_TRUE_COLOR,
                glx::GLX_DRAWABLE_TYPE, glx::GLX_WINDOW_BIT,
                glx::GLX_RENDER_TYPE, glx::GLX_RGBA_BIT,
                glx::GLX_RED_SIZE, config.red_bits as i32,
                glx::GLX_GREEN_SIZE, config.green_bits as i32,
                glx::GLX_BLUE_SIZE, config.blue_bits as i32,
                glx::GLX_ALPHA_SIZE, config.alpha_bits as i32,
                glx::GLX_DEPTH_SIZE, config.depth_bits as i32,
                glx::GLX_STENCIL_SIZE, config.stencil_bits as i32,
                glx::GLX_DOUBLEBUFFER, config.double_buffer as i32,
                glx::GLX_SAMPLE_BUFFERS, config.samples.is_some() as i32,
                glx::GLX_SAMPLES, config.samples.unwrap_or(0) as i32,
                GLX_FRAMEBUFFER_SRGB_CAPABLE_ARB, config.srgb as i32,
                0,
            ];

            let mut n_configs = 0;
            let fb_config =
                unsafe { glx::glXChooseFBConfig(display, screen, fb_attribs.as_ptr(), &mut n_configs) };

            error_handler.check()?;

            if n_configs <= 0 {
                return Err(GlError::CreationFailed(CreationFailedError::InvalidFBConfig));
            }

            #[allow(non_snake_case)]
                let glXCreateContextAttribsARB: GlXCreateContextAttribsARB = unsafe {
                let addr = get_proc_address("glXCreateContextAttribsARB");
                if addr.is_null() {
                    return Err(GlError::CreationFailed(CreationFailedError::GetProcAddressFailed));
                } else {
                    std::mem::transmute(addr)
                }
            };

            #[allow(non_snake_case)]
                let glXSwapIntervalEXT: GlXSwapIntervalEXT = unsafe {
                let addr = get_proc_address("glXSwapIntervalEXT");
                if addr.is_null() {
                    return Err(GlError::CreationFailed(CreationFailedError::GetProcAddressFailed));
                } else {
                    std::mem::transmute(addr)
                }
            };

            error_handler.check()?;

            let profile_mask = match config.profile {
                Profile::Core => glx::arb::GLX_CONTEXT_CORE_PROFILE_BIT_ARB,
                Profile::Compatibility => glx::arb::GLX_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB,
            };

            #[rustfmt::skip]
                let ctx_attribs = [
                glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB, config.version.0 as i32,
                glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB, config.version.1 as i32,
                glx::arb::GLX_CONTEXT_PROFILE_MASK_ARB, profile_mask,
                0,
            ];

            let context = unsafe {
                glXCreateContextAttribsARB(
                    display,
                    *fb_config,
                    std::ptr::null_mut(),
                    1,
                    ctx_attribs.as_ptr(),
                )
            };

            error_handler.check()?;

            if context.is_null() {
                return Err(GlError::CreationFailed(CreationFailedError::ContextCreationFailed));
            }

            unsafe {
                let res = glx::glXMakeCurrent(display, handle.window, context);
                error_handler.check()?;
                if res == 0 {
                    return Err(GlError::CreationFailed(CreationFailedError::MakeCurrentFailed));
                }

                glXSwapIntervalEXT(display, handle.window, config.vsync as i32);
                error_handler.check()?;

                if glx::glXMakeCurrent(display, 0, std::ptr::null_mut()) == 0 {
                    error_handler.check()?;
                    return Err(GlError::CreationFailed(CreationFailedError::MakeCurrentFailed));
                }
            }

            Ok(GlContext {
                window: handle.window,
                display,
                context,
            })
        })
    }

    pub unsafe fn make_current(&self) {
        errors::XErrorHandler::handle(self.display, |error_handler| {
            let res = glx::glXMakeCurrent(self.display, self.window, self.context);
            error_handler.check().unwrap();
            if res == 0 {
                panic!("make_current failed")
            }
        })
    }

    pub unsafe fn make_not_current(&self) {
        errors::XErrorHandler::handle(self.display, |error_handler| {
            let res = glx::glXMakeCurrent(self.display, 0, std::ptr::null_mut());
            error_handler.check().unwrap();
            if res == 0 {
                panic!("make_not_current failed")
            }
        })
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        get_proc_address(symbol)
    }

    pub fn swap_buffers(&self) {
        errors::XErrorHandler::handle(self.display, |error_handler| {
            unsafe {
                glx::glXSwapBuffers(self.display, self.window);
            }
            error_handler.check().unwrap();
        })
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {}
}
