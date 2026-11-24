#![feature(no_core)]
#![no_core]

extern crate base;
fn main() {}

#[cfg(accepted)]
#[unsafe(no_mangle)]
pub extern "C" fn can_unwind(info: &base::PanicInfo<'_>) -> bool {
    info.can_unwind
}

#[cfg(private_message)]
pub fn message(info: &'static base::PanicInfo<'static>) -> &'static base::Arguments<'static> {
    info.message
}

#[cfg(wrong_role)]
pub fn wrong(info: &'static base::Arguments<'static>) -> &'static base::PanicInfo<'static> {
    info
}

#[cfg(handler)]
#[panic_handler]
fn handle(_: &base::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(wrong_handler)]
#[panic_handler]
fn handle_wrong(_: &base::Arguments<'_>) -> ! {
    loop {}
}

#[cfg(duplicate_handler)]
#[panic_handler]
fn first(_: &base::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(duplicate_handler)]
#[panic_handler]
fn second(_: &base::PanicInfo<'_>) -> ! {
    loop {}
}
