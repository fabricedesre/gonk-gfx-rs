/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate android_logger;
extern crate gonk_gfx;
#[macro_use]
extern crate log;

use gonk_gfx::{input, window};
use std::sync::mpsc::channel;

fn main() {
    android_logger::init_once(log::LogLevel::Info);

    info!("About to create window");

    let window = window::Window::new();

    let (sender, receiver) = channel();
    input::run_input_loop(&sender);
    loop {
        let event = receiver.recv().unwrap();
        debug!("Got event {:?}", event);
    }
}
