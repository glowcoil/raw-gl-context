use std::ffi::{c_void, CString, OsStr};
use std::os::windows::ffi::OsStrExt;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use winapi::shared::minwindef::HMODULE;
use winapi::shared::windef::{HDC, HGLRC, HWND};
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};
use winapi::um::wingdi::{
    wglCreateContext, wglDeleteContext, wglGetProcAddress, wglMakeCurrent, ChoosePixelFormat,
    SetPixelFormat, SwapBuffers, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE,
    PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
};
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetDC, RegisterClassW, ReleaseDC, CS_OWNDC,
    CW_USEDEFAULT, WNDCLASSW,
};

use crate::{GlConfig, GlError};

// See https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_create_context.txt
type WglCreateContextAttribsARB = extern "system" fn(HDC, HGLRC, *const i32) -> HGLRC;

const WGL_CONTEXT_MAJOR_VERSION_ARB: i32 = 0x2091;
const WGL_CONTEXT_MINOR_VERSION_ARB: i32 = 0x2092;
const WGL_CONTEXT_PROFILE_MASK_ARB: i32 = 0x9126;

const WGL_CONTEXT_CORE_PROFILE_BIT_ARB: i32 = 0x00000001;

pub struct GlContext {
    hwnd: HWND,
    hdc: HDC,
    hglrc: HGLRC,
    gl_library: HMODULE,
}

impl GlContext {
    pub fn create(
        parent: &impl HasRawWindowHandle,
        config: GlConfig,
    ) -> Result<GlContext, GlError> {
        let handle = if let RawWindowHandle::Windows(handle) = parent.raw_window_handle() {
            handle
        } else {
            return Err(GlError::InvalidWindowHandle);
        };

        if handle.hwnd.is_null() {
            return Err(GlError::InvalidWindowHandle);
        }

        unsafe {
            // Create temporary window and context to load function pointers

            let mut class_name: Vec<u16> =
                OsStr::new("raw-gl-context-window").encode_wide().collect();
            class_name.push(0);

            let wnd_class = WNDCLASSW {
                style: CS_OWNDC,
                lpfnWndProc: Some(DefWindowProcW),
                hInstance: std::ptr::null_mut(),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            };

            // Ignore errors, since class might be registered multiple times
            let window_class = RegisterClassW(&wnd_class);

            let hwnd_tmp = CreateWindowExW(
                0,
                window_class as *const u16,
                class_name.as_ptr(),
                0,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );

            if hwnd_tmp.is_null() {
                return Err(GlError::CreationFailed);
            }

            let hdc_tmp = GetDC(hwnd_tmp);

            let pfd_tmp = PIXELFORMATDESCRIPTOR {
                nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
                nVersion: 1,
                dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                iPixelType: PFD_TYPE_RGBA,
                cColorBits: 32,
                cAlphaBits: 8,
                cDepthBits: 24,
                cStencilBits: 8,
                iLayerType: PFD_MAIN_PLANE,
                ..std::mem::zeroed()
            };

            SetPixelFormat(hdc_tmp, ChoosePixelFormat(hdc_tmp, &pfd_tmp), &pfd_tmp);

            let hglrc_tmp = wglCreateContext(hdc_tmp);
            if hglrc_tmp.is_null() {
                ReleaseDC(hwnd_tmp, hdc_tmp);
                DestroyWindow(hwnd_tmp);
                return Err(GlError::CreationFailed);
            }

            wglMakeCurrent(hdc_tmp, hglrc_tmp);

            #[allow(non_snake_case)]
            let wglCreateContextAttribsARB: WglCreateContextAttribsARB = std::mem::transmute(
                wglGetProcAddress(CString::new("wglCreateContextAttribsARB").unwrap().as_ptr()),
            );

            wglMakeCurrent(hdc_tmp, std::ptr::null_mut());
            ReleaseDC(hwnd_tmp, hdc_tmp);
            DestroyWindow(hwnd_tmp);

            // Create actual context

            let hwnd = handle.hwnd as HWND;

            let hdc = GetDC(hwnd);

            let pfd = PIXELFORMATDESCRIPTOR {
                nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
                nVersion: 1,
                dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                iPixelType: PFD_TYPE_RGBA,
                cColorBits: 32,
                cDepthBits: 24,
                cStencilBits: 8,
                iLayerType: PFD_MAIN_PLANE,
                ..std::mem::zeroed()
            };

            SetPixelFormat(hdc, ChoosePixelFormat(hdc, &pfd), &pfd);

            #[rustfmt::skip]
            let ctx_attribs = [
                WGL_CONTEXT_MAJOR_VERSION_ARB, 3,
                WGL_CONTEXT_MINOR_VERSION_ARB, 2,
                WGL_CONTEXT_PROFILE_MASK_ARB, WGL_CONTEXT_CORE_PROFILE_BIT_ARB,
                0
            ];

            let hglrc = wglCreateContextAttribsARB(hdc, std::ptr::null_mut(), ctx_attribs.as_ptr());
            if hglrc == std::ptr::null_mut() {
                return Err(GlError::CreationFailed);
            }

            let gl_library = LoadLibraryA(CString::new("opengl32.dll").unwrap().as_ptr());

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

    pub fn make_not_current(&self) {
        unsafe {
            wglMakeCurrent(self.hdc, std::ptr::null_mut());
        }
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        let symbol = CString::new(symbol).unwrap();
        let addr = unsafe { wglGetProcAddress(symbol.as_ptr()) as *const c_void };
        if !addr.is_null() {
            addr
        } else {
            unsafe { GetProcAddress(self.gl_library, symbol.as_ptr()) as *const c_void }
        }
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
