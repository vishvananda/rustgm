#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Decoder<'data> {
    bytes: &'data [u8],
}

impl<'data> Decoder<'data> {
    fn checked(&self) -> Option<impl Iterator<Item = u8> + 'data> {
        let mut bytes = self
            .bytes
            .chunks_exact(2)
            .map(|pair| match pair {
                [left, right] => [left, right],
                _ => unreachable!(),
            })
            .map(|[&left, &right]| left ^ right);
        let values = core::iter::from_fn(move || {
            bytes.next().map(|byte| if byte == 0 { Err(()) } else { Ok(byte) })
        });
        if values.clone().any(|value| value.is_err()) {
            None
        } else {
            Some(values.map(Result::unwrap))
        }
    }
}

fn check() -> i32 {
    let bytes = [0_u8, 2, 4, 3];
    let mut values = match (Decoder { bytes: &bytes }).checked() {
        Some(values) => values,
        None => return 1,
    };
    i32::from(values.next() != Some(2) || values.next() != Some(7) || values.next().is_some())
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
