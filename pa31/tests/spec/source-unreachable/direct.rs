#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

unsafe extern "C" {
    fn _Exit(status: i32) -> !;
}

fn choose(value: u8) -> i32 {
    match value {
        1 => 19,
        2 => 42,
        _ => unsafe { core::intrinsics::unreachable() },
    }
}

fn experiment() -> i32 {
    if choose(1) == 19 && choose(2) == 42 {
        unsafe { _Exit(0) }
    }
    3
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    experiment()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(experiment());
}
