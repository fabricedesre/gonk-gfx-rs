/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A wrapper around the hwc device

use gonk_gfx::*;
use hardware::*;
use libc::{c_char, c_int, c_void, size_t};
use std::ffi::CString;
use std::mem::transmute;
use std::ptr;

// From hardware/libhardware/include/hardware/hwcomposer.h

pub const HWC_DISPLAY_NO_ATTRIBUTE: u32 = 0;
pub const HWC_DISPLAY_VSYNC_PERIOD: u32 = 1;
pub const HWC_DISPLAY_WIDTH: u32 = 2;
pub const HWC_DISPLAY_HEIGHT: u32 = 3;
pub const HWC_DISPLAY_DPI_X: u32 = 4;
pub const HWC_DISPLAY_DPI_Y: u32 = 5;

pub const HWC_POWER_MODE_OFF: c_int = 0;
pub const HWC_POWER_MODE_DOZE: c_int = 1;
pub const HWC_POWER_MODE_NORMAL: c_int = 2;
pub const HWC_POWER_MODE_DOZE_SUSPEND: c_int = 3;

#[repr(C)]
pub struct hwc_composer_device {
    pub common: hw_device,
    pub prepare:
        extern "C" fn(*mut hwc_composer_device, size_t, *mut *mut hwc_display_contents) -> c_int,
    pub set:
        extern "C" fn(*mut hwc_composer_device, size_t, *mut *mut hwc_display_contents) -> c_int,
    pub event_control: extern "C" fn(*mut hwc_composer_device, c_int, c_int, c_int) -> c_int,
    pub set_power_mode: extern "C" fn(*mut hwc_composer_device, c_int, c_int) -> c_int,
    pub query: extern "C" fn(*mut hwc_composer_device, c_int, *mut c_int) -> c_int,
    pub register_procs: extern "C" fn(*mut hwc_composer_device, *const hwc_procs),
    pub dump: extern "C" fn(*mut hwc_composer_device, *const c_char, c_int),
    pub get_display_configs:
        extern "C" fn(*mut hwc_composer_device, c_int, *mut u32, *mut size_t) -> c_int,
    pub get_display_attributes:
        extern "C" fn(*mut hwc_composer_device, c_int, u32, *const u32, *mut i32) -> c_int,
    reserved: [*mut c_void; 4],
}

#[repr(C)]
pub struct hwc_color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct hwc_rect {
    pub left: c_int,
    pub top: c_int,
    pub right: c_int,
    pub bottom: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct hwc_frect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[repr(C)]
pub struct hwc_region {
    pub num_rects: i32,
    pub rects: *const hwc_rect,
}

pub const HWC_FRAMEBUFFER: i32 = 0;
pub const HWC_OVERLAY: i32 = 1;
pub const HWC_BACKGROUND: i32 = 2;
pub const HWC_FRAMEBUFFER_TARGET: i32 = 3;
pub const HWC_BLIT: i32 = 4;

pub const HWC_SKIP_LAYER: u32 = 1;

#[repr(C)]
pub struct hwc_layer {
    pub composition_type: i32,
    pub hints: u32,
    pub flags: u32,
    pub handle: *const native_handle,
    pub transform: u32,
    pub blending: i32,
    pub source_crop: hwc_frect, // If HWC 1.3, then this takes floats
    pub display_frame: hwc_rect,
    pub visible_region_screen: hwc_region,
    pub acquire_fence_fd: c_int,
    pub release_fence_fd: c_int,
    pub plane_alpha: u8,
    pub pad: [u8; 3],
    pub surface_damage: hwc_region,
    pub reserved: [u8; (96 - 84)],
}

#[repr(C)]
pub struct hwc_display_contents {
    pub retire_fence_fd: c_int,
    pub outbuf: *const u32,
    pub outbuf_acquire_fence_fd: c_int,
    pub flags: u32,
    pub num_hw_layers: size_t,
    pub hw_layers: [hwc_layer; 2],
}

#[repr(C)]
pub struct hwc_procs {
    invalidate: extern "C" fn(*const hwc_procs),
    vsync: extern "C" fn(*const hwc_procs, c_int, i64),
    hotplug: extern "C" fn(*const hwc_procs, c_int, c_int),
}

#[derive(PartialEq)]
pub enum HwcApiVersion {
    Hwc1_3,
    Hwc1_4,
}

impl HwcApiVersion {
    pub fn hwc_api_version(maj: u32, min: u32) -> u32 {
        // HARDWARE_MAKE_API_VERSION_2, from Android hardware.h
        (((maj & 0xff) << 24) | ((min & 0xff) << 16) | (1 & 0xffff))
    }

    pub fn from(version: u32) -> Option<HwcApiVersion> {
        if HwcApiVersion::hwc_api_version(1, 3) == version {
            Some(HwcApiVersion::Hwc1_3)
        } else if HwcApiVersion::hwc_api_version(1, 4) == version {
            Some(HwcApiVersion::Hwc1_4)
        } else {
            None
        }
    }
}

pub struct HwcDevice {
    native: *mut hwc_composer_device,
    version: HwcApiVersion,
}

impl HwcDevice {
    pub fn new() -> Option<HwcDevice> {
        let mut hwc_mod = ptr::null();
        unsafe {
            let cstr = CString::new("hwcomposer").unwrap();
            let ret = hw_get_module(cstr.as_ptr(), &mut hwc_mod);
            if ret != 0 {
                error!("Failed to get HWC module!");
                return None;
            }
        }

        let hwc_device: *mut hwc_composer_device;
        unsafe {
            let mut device = ptr::null();
            let cstr = CString::new("composer").unwrap();
            let ret = ((*(*hwc_mod).methods).open)(hwc_mod, cstr.as_ptr(), &mut device);
            if ret != 0 {
                error!("Failed to get HWC device!");
                return None;
            }
            hwc_device = transmute(device);

            match HwcApiVersion::from((*hwc_device).common.version) {
                None => None,
                Some(version) => Some(HwcDevice {
                    native: hwc_device,
                    version,
                }),
            }
        }
    }

    pub fn get_dimensions(&self) -> (i32, i32) {
        let attrs: [u32; 3] = [
            HWC_DISPLAY_WIDTH,
            HWC_DISPLAY_HEIGHT,
            HWC_DISPLAY_NO_ATTRIBUTE,
        ];
        let mut values: [i32; 3] = [0; 3];
        unsafe {
            // In theory, we should check the return code.
            // However, there are HALs which implement this wrong.
            let _ = ((*self.native).get_display_attributes)(
                self.native,
                0,
                0,
                attrs.as_ptr(),
                values.as_mut_ptr(),
            );
        }
        (values[0], values[1])
    }

    pub fn set_display(&self, enable: bool) {
        if enable {
            unsafe {
                autosuspend_disable();
            }
        }

        // If the version is 1.3, we actually are using the blank()
        // method behing the scene.
        let mode = if self.version == HwcApiVersion::Hwc1_3 {
            if enable {
                0
            } else {
                1
            }
        } else {
            if enable {
                HWC_POWER_MODE_NORMAL
            } else {
                HWC_POWER_MODE_OFF
            }
        };
        unsafe {
            ((*self.native).set_power_mode)(self.native, 0, mode);
        }
    }

    pub fn native(&self) -> *mut hwc_composer_device {
        self.native
    }
}
