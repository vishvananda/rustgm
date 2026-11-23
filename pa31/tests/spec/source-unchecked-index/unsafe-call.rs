#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn experiment() -> i32 {
    let values = [7_u8, 11];
    let slice: &[u8] = &values;
    let _read = slice.get_unchecked(0);
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
