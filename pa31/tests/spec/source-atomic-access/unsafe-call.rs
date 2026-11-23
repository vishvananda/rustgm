#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::{AtomicOrdering, atomic_store};

fn experiment() -> i32 {
    let mut value = 0u32;
    atomic_store::<_, { AtomicOrdering::Relaxed }>(&raw mut value, 7);
    0
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
    let _ = experiment();
}
