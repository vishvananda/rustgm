#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn nonzero() -> i32 {
    let data = [8_u32, 13, 21];
    let first = (&raw const data).cast::<u32>();
    // Both offsets stay in the same allocation and have the required order.
    let last = unsafe { first.add(2) };
    i32::from(unsafe { first.offset_from(last) != -2 || last.offset_from_unsigned(first) != 2 })
}

fn check(argc: i32) -> i32 {
    let pointee = ();
    let pointer = &raw const pointee;
    // Each zero-sized call uses the same address for both pointers.
    match argc {
        1 => nonzero(),
        2 => unsafe { pointer.offset_from(pointer) as i32 },
        _ => unsafe { pointer.offset_from_unsigned(pointer) as i32 },
    }
}

#[cfg(guest_probe)]
#[link(name = "c")]
unsafe extern "C" {
    fn _Exit(status: i32) -> !;
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    // Terminate the standalone process when the source panic hook is called.
    unsafe { _Exit(23) }
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    check(argc)
}

#[cfg(not(guest_probe))]
fn main() {
    std::panic::set_hook(Box::new(|_| std::process::exit(23)));
    std::process::exit(check(std::env::args_os().len() as i32));
}
