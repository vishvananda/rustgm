#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct View<'a>(&'a mut u8);

fn shorten<'a: 'b, 'b>(view: View<'a>, limit: &'b u8) -> Result<View<'b>, &'static str> {
    Ok(view).and_then(|view| -> Result<View<'b>, &'static str> {
        if *view.0 > *limit {
            return Err("limit");
        }
        *view.0 += 1;
        Ok(view)
    })
}

fn check() -> i32 {
    let mut value = 3;
    let limit = 5;
    if shorten(View(&mut value), &limit).map(|view| *view.0) != Ok(4) {
        return 1;
    }
    i32::from(value != 4)
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
