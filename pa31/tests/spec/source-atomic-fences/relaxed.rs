#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn experiment() {
    unsafe { core::intrinsics::atomic_fence::<{ core::intrinsics::AtomicOrdering::Relaxed }>() }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    experiment();
    0
}

#[cfg(not(guest_probe))]
fn main() {
    experiment();
}
