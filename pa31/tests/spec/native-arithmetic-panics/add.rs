#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn checked(value: u8) -> u8 {
    value + 1
}

#[cfg(guest_probe)]
#[link(name = "c")]
unsafe extern "C" {
    fn _Exit(status: i32) -> !;
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let status = if info.location().is_some_and(|site| site.line() == 5 && site.column() == 5) {
        73
    } else {
        74
    };
    unsafe { _Exit(status) }
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    if argc == 1 {
        if checked(4) == 5 { 0 } else { 1 }
    } else {
        i32::from(checked(u8::MAX))
    }
}

#[cfg(not(guest_probe))]
fn main() {
    if std::env::args_os().nth(1).is_none() {
        assert_eq!(checked(4), 5);
    } else {
        std::panic::set_hook(Box::new(|info| {
            let status = if info.location().is_some_and(|site| site.line() == 5 && site.column() == 5) {
                73
            } else {
                74
            };
            std::process::exit(status);
        }));
        let _ = checked(u8::MAX);
    }
}
