/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using Gonk interfaces.

use egl::{self, EGLContext, EGLDisplay, EGLSurface};
use gleam::gl::{self, Gl};
use std::ffi::CString;
use std::mem::transmute;
use std::ptr;
use std::rc::Rc;
use gonk_gfx::*;

/// The type of a window.
pub struct Window {
    pub width: u32,
    pub height: u32,
    pub native_window: *mut GonkNativeWindow,
    pub dpy: EGLDisplay,
    pub ctx: EGLContext,
    pub surf: EGLSurface,
    pub gl: Rc<Gl>,
}

impl Window {
    /// Creates a new window.
    pub fn new() -> Rc<Window> {
        let mut hwc_mod = ptr::null();
        unsafe {
            let cstr = CString::new("hwcomposer").unwrap();
            let ret = hw_get_module(cstr.as_ptr(), &mut hwc_mod);
            assert!(ret == 0, "Failed to get HWC module!");
        }

        let hwc_device: *mut hwc_composer_device;
        unsafe {
            let mut device = ptr::null();
            let cstr = CString::new("composer").unwrap();
            let ret = ((*(*hwc_mod).methods).open)(hwc_mod, cstr.as_ptr(), &mut device);
            assert!(ret == 0, "Failed to get HWC device!");
            hwc_device = transmute(device);
            // Require HWC 1.1 or newer
            // XXX add HAL version function/macro
            assert!((*hwc_device).common.version > (1 << 8), "HWC too old!");
        }

        let attrs: [u32; 4] = [
            HWC_DISPLAY_WIDTH,
            HWC_DISPLAY_HEIGHT,
            HWC_DISPLAY_DPI_X,
            HWC_DISPLAY_NO_ATTRIBUTE,
        ];
        let mut values: [i32; 4] = [0, 0, 0, 0];
        unsafe {
            // In theory, we should check the return code.
            // However, there are HALs which implement this wrong.
            let _ = ((*hwc_device).get_display_attributes)(
                hwc_device,
                0,
                0,
                attrs.as_ptr(),
                values.as_mut_ptr(),
            );
        }

        let mut gralloc_mod = ptr::null();
        let alloc_dev: *mut alloc_device;
        unsafe {
            let mut device = ptr::null();
            let cstr = CString::new("gralloc").unwrap();
            let ret1 = hw_get_module(cstr.as_ptr(), &mut gralloc_mod);
            assert_eq!(ret1, 0, "Failed to get gralloc module!");
            let cstr2 = CString::new("gpu0").unwrap();
            let ret2 = ((*(*gralloc_mod).methods).open)(gralloc_mod, cstr2.as_ptr(), &mut device);
            assert_eq!(ret2, 0, "Failed to get gralloc module!");
            alloc_dev = transmute(device);
        }

        let width = values[0];
        let height = values[1];
        let dpy = egl::get_display(egl::EGL_DEFAULT_DISPLAY).unwrap();

        let mut major: i32 = 0;
        let mut minor: i32 = 0;
        let ret = { egl::initialize(dpy, &mut major, &mut minor) };

        assert!(ret, "Failed to initialize EGL!");

        info!("EGL initialized {}.{}", major, minor);

        let conf_attr = [
            egl::EGL_SURFACE_TYPE,
            egl::EGL_WINDOW_BIT,
            egl::EGL_RENDERABLE_TYPE,
            egl::EGL_OPENGL_ES2_BIT,
            egl::EGL_RED_SIZE,
            6,
            egl::EGL_GREEN_SIZE,
            5,
            egl::EGL_BLUE_SIZE,
            6,
            egl::EGL_ALPHA_SIZE,
            0,
            egl::EGL_NONE,
        ];

        let config = egl::choose_config(dpy, &conf_attr, 1);

        assert!(config.is_some(), "Failed to choose a config");
        let config = config.unwrap();

        info!("Creating {}x{} native window", width, height);

        let usage = GRALLOC_USAGE_HW_FB | GRALLOC_USAGE_HW_RENDER | GRALLOC_USAGE_HW_COMPOSER;
        let native_window = GonkNativeWindow::new(alloc_dev, hwc_device, width, height, usage);

        let eglwindow =
            unsafe { egl::create_window_surface(dpy, config, transmute(native_window), &[]) };

        assert!(eglwindow.is_some());
        let eglwindow = eglwindow.unwrap();

        let ctx_attr = [egl::EGL_CONTEXT_CLIENT_VERSION, 2, egl::EGL_NONE, 0];

        let ctx =
            unsafe { egl::create_context(dpy, config, transmute(egl::EGL_NO_CONTEXT), &ctx_attr) };

        assert!(ctx.is_some(), "Failed to create a context!");
        let ctx = ctx.unwrap();

        let ret = egl::make_current(dpy, eglwindow, eglwindow, ctx);
        assert!(ret, "Failed to make current!");

        unsafe {
            (*native_window).alloc_buffers();
            autosuspend_disable();
            ((*hwc_device).set_power_mode)(hwc_device, 0, HWC_POWER_MODE_NORMAL);
        }

        let gl = unsafe { gl::GlesFns::load_with(|s| egl::get_proc_address(s) as *const _) };

        gl.clear_color(1f32, 0f32, 0f32, 1f32);
        // gl.clear(gl::COLOR_BUFFER_BIT);
        egl::swap_buffers(dpy, eglwindow);

        // Create our window object.
        let window = Window {
            width: width as u32,
            height: height as u32,
            native_window: native_window,
            dpy: dpy,
            ctx: ctx,
            surf: eglwindow,
            gl: gl,
        };

        Rc::new(window)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            ((*self.native_window).window.common.dec_ref)(&mut (*self.native_window).window.common);
        }
    }
}
