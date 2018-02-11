/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use gonk_gfx::native_handle;
use hardware::*;
use libc::{c_char, c_int, c_void, size_t};
use std::ffi::CString;
use std::mem::transmute;
use std::ptr;

// From system/core/include/system/graphics.h

#[repr(C)]
pub struct android_ycbcr {
    y: *mut c_void,
    cb: *mut c_void,
    cr: *mut c_void,
    ystride: size_t,
    cstride: size_t,
    chroma_step: size_t,
    reserved: [u32; 8],
}

// From hardware/libhardware/include/hardware/gralloc.h

#[repr(C)]
pub struct gralloc_module {
    common: hw_module,
    register_buffer: extern "C" fn(*const gralloc_module, *const native_handle) -> c_int,
    unregister_buffer: extern "C" fn(*const gralloc_module, *const native_handle) -> c_int,
    lock: extern "C" fn(
        *const gralloc_module,
        *const native_handle,
        c_int,
        c_int,
        c_int,
        c_int,
        *mut *mut c_void,
    ) -> c_int,
    unlock: extern "C" fn(*const gralloc_module, *const native_handle) -> c_int,
    perform: extern "C" fn(*const gralloc_module, c_int, ...) -> c_int,
    lock_ycbcr: extern "C" fn(
        *const gralloc_module,
        *const native_handle,
        c_int,
        c_int,
        c_int,
        c_int,
        c_int,
        *mut android_ycbcr,
    ) -> c_int,
    reserved: [*mut c_void; 6],
}

#[repr(C)]
pub struct alloc_device {
    common: hw_device,
    pub alloc: extern "C" fn(
        *mut alloc_device,
        c_int,
        c_int,
        c_int,
        c_int,
        *mut *const native_handle,
        *mut c_int,
    ) -> c_int,
    pub free: extern "C" fn(*mut alloc_device, *const native_handle) -> c_int,
    pub dump: Option<extern "C" fn(*mut alloc_device, *mut c_char, c_int)>,
    reserved: [*mut c_void; 7],
}

pub fn get_gralloc_module() -> *mut alloc_device {
    let mut gralloc_mod = ptr::null();
    let alloc_dev: *mut alloc_device;
    unsafe {
        let mut device = ptr::null();
        let cstr = CString::new("gralloc").unwrap();
        let ret1 = hw_get_module(cstr.as_ptr(), &mut gralloc_mod);
        assert_eq!(ret1, 0, "Failed to get gralloc module!");
        let cstr2 = CString::new("gpu0").unwrap();
        let ret2 = ((*(*gralloc_mod).methods).open)(gralloc_mod, cstr2.as_ptr(), &mut device);
        assert_eq!(ret2, 0, "Failed to open gpu0 on gralloc module!");
        alloc_dev = transmute(device);
    }
    alloc_dev
}
