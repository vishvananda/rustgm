#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

#[track_caller]
fn remainder(value: i8, modulus: i8) -> i8 {
    value % modulus
}

fn probe(argc: i32) -> i32 {
    match argc {
        1 => match remainder(13, 4) {
            1 => 0,
            _ => 71,
        },
        2 => {
            let _ = remainder(5, 0);
            72
        }
        _ => {
            let _ = remainder(i8::MIN, -1);
            72
        }
    }
}

fn outcome(line: u32, column: u32, message: Option<&str>) -> i32 {
    match (line, column, message) {
        (16, 21, Some("attempt to calculate the remainder with a divisor of zero")) => 73,
        (20, 21, Some("attempt to calculate the remainder with overflow")) => 75,
        _ => 74,
    }
}

#[cfg(guest_probe)]
#[link(name = "c")]
unsafe extern "C" {
    fn _Exit(status: i32) -> !;
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let status = match info.location() {
        Some(location) => outcome(location.line(), location.column(), info.message().as_str()),
        None => 74,
    };
    unsafe { _Exit(status) }
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    probe(argc)
}

#[cfg(not(guest_probe))]
fn main() {
    std::panic::set_hook(Box::new(|info| {
        let message = info.payload().downcast_ref::<&str>().copied();
        let status = match info.location() {
            Some(location) => outcome(location.line(), location.column(), message),
            None => 74,
        };
        std::process::exit(status);
    }));
    std::process::exit(probe(std::env::args_os().count() as i32));
}
