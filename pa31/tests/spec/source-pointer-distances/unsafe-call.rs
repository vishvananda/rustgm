#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn experiment() -> i32 {
    let value = 7_u32;
    let pointer = &raw const value;
    pointer.offset_from_unsigned(pointer) as i32
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
