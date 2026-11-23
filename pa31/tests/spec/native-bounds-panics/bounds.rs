#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

fn array(index: usize) -> u8 {
    [3, 5, 8][index]
}

#[track_caller]
fn slice(bytes: &[u8], index: usize) -> u8 {
    bytes[index]
}

fn check(argc: i32) -> i32 {
    match argc {
        1 if array(2) == 8 && slice(&[9, 7], 1) == 7 => 0,
        1 => 1,
        2 => i32::from(array(9)),
        _ => i32::from(slice(&[13, 21], 6)),
    }
}

#[cfg(guest_probe)]
#[link(name = "c")]
unsafe extern "C" {
    fn _Exit(status: i32) -> !;
    fn write(fd: i32, source: *const u8, length: usize) -> isize;
}

#[cfg(guest_probe)]
struct Output;

#[cfg(guest_probe)]
impl core::fmt::Write for Output {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let length = text.len();
        // The string stays live for this write to the inherited stdout.
        let wrote = unsafe { write(1, text.as_ptr(), length) };
        if wrote == length as isize {
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }
}

fn location_status(line: u32, column: u32) -> i32 {
    match (line, column) {
        (5, 5) => 73,
        (18, 24) => 75,
        _ => 74,
    }
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let status = match info.location() {
        Some(site) => location_status(site.line(), site.column()),
        None => 74,
    };
    let printed = core::fmt::write(&mut Output, format_args!("{}", info.message())).is_ok();
    unsafe { _Exit(if printed { status } else { 76 }) }
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, _argv: *const *const u8) -> i32 {
    check(argc)
}

#[cfg(not(guest_probe))]
fn main() {
    std::panic::set_hook(Box::new(|info| {
        let status = match info.location() {
            Some(site) => location_status(site.line(), site.column()),
            None => 74,
        };
        if let Some(text) = info.payload().downcast_ref::<String>() {
            print!("{text}");
        } else if let Some(text) = info.payload().downcast_ref::<&str>() {
            print!("{text}");
        }
        let _ = std::io::Write::flush(&mut std::io::stdout());
        std::process::exit(status);
    }));
    std::process::exit(check(std::env::args_os().len() as i32));
}
