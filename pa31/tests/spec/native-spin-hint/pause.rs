#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn pause() {
    core::hint::spin_loop();
    core::arch::x86_64::_mm_pause();
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    pause();
    0
}

#[cfg(not(guest_probe))]
fn main() {
    pause();
}
