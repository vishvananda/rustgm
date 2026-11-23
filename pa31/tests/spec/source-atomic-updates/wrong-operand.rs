#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::intrinsics::{AtomicOrdering, atomic_xadd};

fn experiment() -> i16 {
    let mut word = 3_i16;
    unsafe { atomic_xadd::<_, _, { AtomicOrdering::Relaxed }>(&raw mut word, 5_u16) }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    experiment() as i32
}

#[cfg(not(guest_probe))]
fn main() {
    let _ = experiment();
}
