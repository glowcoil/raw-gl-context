use std::ffi::{c_void, CString};

use raw_window_handle::RawWindowHandle;

use winapi::shared::minwindef::HMODULE;
use winapi::shared::windef::{HDC, HGLRC, HWND};
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};
use winapi::um::wingdi::{
    wglCreateContext, wglDeleteContext, wglMakeCurrent, ChoosePixelFormat, SetPixelFormat,
    SwapBuffers, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE, PFD_SUPPORT_OPENGL,
    PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
};
use winapi::um::winuser::{GetDC, ReleaseDC};

pub struct GlContext {
    hwnd: HWND,
    hdc: HDC,
    hglrc: HGLRC,
    gl_library: HMODULE,
}

impl GlContext {
    pub fn create(raw_window_handle: RawWindowHandle) -> Result<GlContext, ()> {
        let handle = if let RawWindowHandle::Windows(handle) = raw_window_handle {
            handle
        } else {
            return Err(());
        };

        unsafe {
            let hwnd = handle.hwnd as HWND;

            let hdc = GetDC(hwnd);

            let mut pfd: PIXELFORMATDESCRIPTOR = std::mem::zeroed();
            pfd.nSize = std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
            pfd.nVersion = 1;
            pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
            pfd.iPixelType = PFD_TYPE_RGBA;
            pfd.cColorBits = 32;
            pfd.cDepthBits = 24;
            pfd.cStencilBits = 8;
            pfd.iLayerType = PFD_MAIN_PLANE;

            let pf_id: i32 = ChoosePixelFormat(hdc, &pfd);
            if pf_id == 0 {
                return Err(());
            }

            if SetPixelFormat(hdc, pf_id, &pfd) == 0 {
                return Err(());
            }

            let hglrc = wglCreateContext(hdc);
            if hglrc == 0 as HGLRC {
                return Err(());
            }

            let gl_library = LoadLibraryA("opengl32.dll\0".as_ptr() as *const i8);

            Ok(GlContext {
                hwnd,
                hdc,
                hglrc,
                gl_library,
            })
        }
    }

    pub fn make_current(&self) {
        unsafe {
            wglMakeCurrent(self.hdc, self.hglrc);
        }
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        let symbol = CString::new(symbol).unwrap();
        unsafe { GetProcAddress(self.gl_library, symbol.as_ptr()) as *const c_void }
    }

    pub fn swap_buffers(&self) {
        unsafe {
            SwapBuffers(self.hdc);
        }
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            wglMakeCurrent(std::ptr::null_mut(), std::ptr::null_mut());
            wglDeleteContext(self.hglrc);
            ReleaseDC(self.hwnd, self.hdc);
        }
    }
}
