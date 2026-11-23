#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Hex<'s> {
    nibbles: &'s [u8],
}

impl<'s> Hex<'s> {
    fn decoded(&self) -> Option<impl Iterator<Item = u8> + 's> {
        let mut pairs = core::iter::once(self.nibbles)
            .map(|slice| match slice {
                [high, low] => {
                    let temporary = *high;
                    [&temporary, low]
                }
                _ => loop {},
            })
            .map(|[&high, &low]| high ^ low);
        Some(core::iter::from_fn(move || pairs.next()))
    }
}

fn experiment() -> i32 {
    let bytes = [2, 3];
    let hex = Hex { nibbles: &bytes };
    let Some(mut output) = hex.decoded() else {
        return 1;
    };
    let Some(first) = output.next() else {
        return 2;
    };
    if first == 1 && output.next().is_none() {
        0
    } else {
        4
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
    experiment()
}

#[cfg(not(guest_probe))]
fn main() {
    assert_eq!(experiment(), 0);
}
