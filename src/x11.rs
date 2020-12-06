use std::ffi::{c_void, CString};
use std::os::raw::{c_int, c_ulong};

use raw_window_handle::RawWindowHandle;

use x11::glx;
use x11::xlib;

type GlXCreateContextAttribsARBProc = unsafe extern "C" fn(
    dpy: *mut xlib::Display,
    fbc: glx::GLXFBConfig,
    share_context: glx::GLXContext,
    direct: xlib::Bool,
    attribs: *const c_int,
) -> glx::GLXContext;

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
    pub fn create(raw_window_handle: RawWindowHandle) -> Result<GlContext, ()> {
        let handle = if let RawWindowHandle::Xlib(handle) = raw_window_handle {
            handle
        } else {
            return Err(());
        };

        let display = handle.display as *mut xlib::_XDisplay;

        let screen = unsafe { xlib::XDefaultScreen(display) };

        #[rustfmt::skip]
        let fb_attribs = [
            glx::GLX_X_RENDERABLE,  1,
            glx::GLX_X_VISUAL_TYPE, glx::GLX_TRUE_COLOR,
            glx::GLX_DRAWABLE_TYPE, glx::GLX_WINDOW_BIT,
            glx::GLX_RENDER_TYPE,   glx::GLX_RGBA_BIT,
            glx::GLX_RED_SIZE,      8,
            glx::GLX_GREEN_SIZE,    8,
            glx::GLX_BLUE_SIZE,     8,
            glx::GLX_ALPHA_SIZE,    8,
            glx::GLX_DEPTH_SIZE,    24,
            glx::GLX_STENCIL_SIZE,  8,
            glx::GLX_DOUBLEBUFFER,  1,
            0,
        ];

        let mut n_configs = 0;
        let fb_config =
            unsafe { glx::glXChooseFBConfig(display, screen, fb_attribs.as_ptr(), &mut n_configs) };

        if n_configs <= 0 {
            return Err(());
        }

        #[rustfmt::skip]
        let ctx_attribs = [
            glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB, 3,
            glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB, 2,
            glx::arb::GLX_CONTEXT_PROFILE_MASK_ARB, glx::arb::GLX_CONTEXT_CORE_PROFILE_BIT_ARB,
            0,
        ];

        let glXCreateContextAttribsARB: GlXCreateContextAttribsARBProc =
            unsafe { std::mem::transmute(get_proc_address("glXCreateContextAttribsARB")) };

        let context = unsafe {
            glXCreateContextAttribsARB(
                display,
                *fb_config,
                std::ptr::null_mut(),
                1,
                ctx_attribs.as_ptr(),
            )
        };

        if context.is_null() {
            return Err(());
        }

        Ok(GlContext {
            window: handle.window,
            display,
            context,
        })
    }

    pub fn make_current(&self) {
        unsafe {
            glx::glXMakeCurrent(self.display, self.window, self.context);
        }
    }

    pub fn make_not_current(&self) {
        unsafe {
            glx::glXMakeCurrent(self.display, 0, std::ptr::null_mut());
        }
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        get_proc_address(symbol)
    }

    pub fn swap_buffers(&self) {
        unsafe {
            glx::glXSwapBuffers(self.display, self.window);
        }
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {}
}
