#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn local_item() -> &'static u16 {
    let data = [7_u16, 11];
    let mut cursor = data.iter().peekable();
    match cursor.peek() {
        Some(item) => *item,
        None => &0,
    }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    i32::from(*local_item())
}

#[cfg(not(guest_probe))]
fn main() {
    let _ = local_item();
}
