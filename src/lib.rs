/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// For size_of::<linux_input_event>() in input.rs
#![feature(const_size_of)]

extern crate egl;
extern crate errno;
extern crate euclid;
extern crate gleam;
extern crate libc;
#[macro_use]
extern crate log;

pub mod gonk_gfx;
pub mod gralloc;
pub mod hardware;
pub mod hwc;
pub mod window;
