#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn escape(value: &u8, limit: &u8) -> Result<&'static u8, &'static str> {
    Ok(value).and_then(|value| {
        if value > limit {
            Err("limit")
        } else {
            Ok(value)
        }
    })
}

fn check() -> i32 {
    let value = 1;
    i32::from(escape(&value, &2).is_err())
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    check()
}

#[cfg(not(guest_probe))]
fn main() {
    std::process::exit(check());
}
