/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using Gonk interfaces.

use egl::{self, EGLContext, EGLDisplay, EGLSurface};
use hwc::HwcDevice;
use gleam::gl::{self, Gl};
use gonk_gfx::*;
use std::mem::transmute;
use std::rc::Rc;

/// The type of a window.
pub struct Window {
    pub width: i32,
    pub height: i32,
    pub dpi: i32,
    hwc: HwcDevice,
    pub native_window: *mut GonkNativeWindow,
    pub dpy: EGLDisplay,
    pub ctx: EGLContext,
    pub surf: EGLSurface,
    pub gl: Rc<Gl>,
}

impl Window {
    /// Creates a new window.
    pub fn new() -> Rc<Window> {
        let hwc = HwcDevice::new();
        assert!(hwc.is_some(), "Failed to get the HWC device");
        let hwc = hwc.unwrap();

        let (width, height, dpi) = hwc.get_dimensions_and_dpi();

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
            8,
            egl::EGL_GREEN_SIZE,
            8,
            egl::EGL_BLUE_SIZE,
            8,
            egl::EGL_ALPHA_SIZE,
            8,
            egl::EGL_DEPTH_SIZE,
            24,
            egl::EGL_NONE,
        ];

        let config = egl::choose_config(dpy, &conf_attr, 1);

        assert!(config.is_some(), "Failed to choose a config");
        let config = config.unwrap();

        info!("Creating {}x{} native window", width, height);

        let usage = GRALLOC_USAGE_HW_FB | GRALLOC_USAGE_HW_RENDER | GRALLOC_USAGE_HW_COMPOSER;
        let native_window = GonkNativeWindow::new(hwc.native(), width, height, usage);

        let surf =
            unsafe { egl::create_window_surface(dpy, config, transmute(native_window), &[]) };

        assert!(surf.is_some());
        let surf = surf.unwrap();

        let ctx_attr = [egl::EGL_CONTEXT_CLIENT_VERSION, 2, egl::EGL_NONE];

        let ctx = egl::create_context(dpy, config, egl::EGL_NO_CONTEXT, &ctx_attr);

        assert!(ctx.is_some(), "Failed to create a context!");
        let ctx = ctx.unwrap();

        let ret = egl::make_current(dpy, surf, surf, ctx);
        assert!(ret, "Failed to make current!");

        unsafe {
            (*native_window).alloc_buffers();
        }
        hwc.set_display(true);

        let gl = unsafe { gl::GlesFns::load_with(|s| egl::get_proc_address(s) as *const _) };

        gl.viewport(0, 0, width, height);

        // Create our window object.
        let window = Window {
            width,
            height,
            dpi,
            hwc,
            native_window,
            dpy,
            ctx,
            surf,
            gl,
        };

        Rc::new(window)
    }

    pub fn fill_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.gl.clear_color(r, g, b, a);
        self.gl.clear(gl::COLOR_BUFFER_BIT);
        egl::swap_buffers(self.dpy, self.surf);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        info!("Dropping Window");
        unsafe {
            ((*self.native_window).window.common.dec_ref)(&mut (*self.native_window).window.common);
        }
        self.hwc.set_display(false);
    }
}
