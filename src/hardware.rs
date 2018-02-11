/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// hardware/libhardware/include/hardware/hardware.h

use libc::{c_char, c_int};

#[repr(C)]
pub struct hw_module_methods {
    pub open: extern "C" fn(*const hw_module, *const c_char, *mut *const hw_device) -> c_int,
}

#[repr(C)]
pub struct hw_module {
    tag: u32,
    module_api_version: u16,
    hal_api_version: u16,
    id: *const c_char,
    name: *const c_char,
    author: *const c_char,
    pub methods: *mut hw_module_methods,
    dso: *mut u32,
    reserved: [u32; (32 - 7)],
}

#[repr(C)]
pub struct hw_device {
    tag: u32,
    pub version: u32,
    module: *mut hw_module,
    reserved: [u32; 12],
    close: extern "C" fn(*mut hw_device) -> c_int,
}

#[link(name = "hardware")]
extern "C" {
    pub fn hw_get_module(id: *const c_char, module: *mut *const hw_module) -> c_int;
}
