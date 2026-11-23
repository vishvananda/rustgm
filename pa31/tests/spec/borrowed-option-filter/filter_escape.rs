#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn retain<'a>(value: &'a u32, comparison: &u32) -> Option<&'static u32> {
    Some(value).filter(|&candidate| *candidate == *comparison)
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn check() -> i32 {
    let value = 1;
    i32::from(retain(&value, &value).is_none())
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    check()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(check());
}
