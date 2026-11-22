#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

struct Invariant<'a>(&'a mut &'a [u8]);

impl PartialEq for Invariant<'_> {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

fn identity<T>(value: T) -> T {
    value
}

fn invalid<'a, 'b>(left: Option<Invariant<'a>>, right: Option<Invariant<'b>>) -> bool {
    left == identity(right)
}

#[cfg(guest_probe)]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}

#[cfg(guest_probe)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    0
}

#[cfg(not(guest_probe))]
fn main() {}
