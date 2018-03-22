/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate android_logger;
extern crate gonk_gfx;
#[macro_use]
extern crate log;

use gonk_gfx::window;
use android_logger::Filter;

#[allow(deprecated)] // for sleep_ms()
fn main() {
    android_logger::init_once(Filter::default().with_min_level(log::Level::Info));

    info!("About to create window");

    let window = window::Window::new();
    window.fill_color(0.0, 1.0, 1.0, 1.0);

    loop {
        ::std::thread::sleep_ms(100000);
    }
}
