#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Reader<'a> {
    rows: &'a [u8],
    cutoff: u8,
}

impl Reader<'_> {
    fn retained(&self) -> impl Iterator<Item = u8> + '_ {
        self.rows
            .iter()
            .filter_map(move |&row| (row != self.cutoff).then_some(row))
    }
}

fn escape(reader: &Reader<'_>) -> impl Iterator<Item = u8> + 'static {
    let rows = reader.retained().map(|row| row + 1);
    let _ = rows.size_hint();
    rows
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

fn check() -> i32 {
    let cutoff = 3;
    let reader = Reader {
        rows: &[2, 3, 5],
        cutoff,
    };
    i32::from(escape(&reader).next().is_none())
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
