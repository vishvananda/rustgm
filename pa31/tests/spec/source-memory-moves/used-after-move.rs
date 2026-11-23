#![feature(core_intrinsics)]
#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Payload(u8);

fn experiment() -> i32 {
    let mut destination = Payload(11);
    let replacement = Payload(23);
    let pointer = &raw mut destination;
    unsafe { core::intrinsics::write_via_move(pointer, replacement) };
    replacement.0 as i32
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
