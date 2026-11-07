#![feature(no_core)]
#![no_core]
extern crate gates;
#[unsafe(no_mangle)]
pub extern "C" fn invoked_line() -> u32 {
    gates::line_wrapper!()
}
#[unsafe(no_mangle)]
pub extern "C" fn invoked_column() -> u32 {
    gates::wrapper!()
}
#[unsafe(no_mangle)]
pub extern "C" fn configured() -> bool {
    gates::cfg!(target_os = "linux")
}
