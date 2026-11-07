#![feature(no_core)]
#![no_core]
extern crate apis;
#[unsafe(no_mangle)]
pub extern "C" fn invoked_line() -> u32 {
    apis::line!()
}
#[unsafe(no_mangle)]
pub extern "C" fn invoked_column() -> u32 {
    apis::api!()
}
#[unsafe(no_mangle)]
pub extern "C" fn configured() -> bool {
    apis::cfg!(target_os = "linux")
}
