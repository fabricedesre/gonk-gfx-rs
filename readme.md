# Basic gfx and input for gonk devices in Rust

To build:
- Install a [Rust toolchain](https://rustup.rs) with the `armv7-linux-androideabi` target (run `rustup target add armv7-linux-androideabi`).
- You need a device build ready, and to set the `GONK_DIR` and `GONK_PRODUCT_NAME` environment variables to the appropriate value. 
- Build with `./build.sh --release --example demo`.
- Push to your device with `adb push target/armv7-linux-androideabi/release/demo /data/local/demo`.
- Run on device.

There will be some logging showing up in `adb logcat`.