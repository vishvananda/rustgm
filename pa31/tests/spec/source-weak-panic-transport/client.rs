#![feature(no_core)]
#![no_core]
extern crate base;
#[panic_handler]
fn handler(_: &base::PanicInfo<'_>) -> ! { loop {} }
#[unsafe(no_mangle)]
pub extern "C" fn use_import(info: &base::PanicInfo<'_>) -> ! {
    base::forward(info)
}
#[unsafe(no_mangle)]
pub extern "C" fn use_second_import(info: &base::PanicInfo<'_>) -> ! {
    base::forward_again(info)
}
fn main() {}
