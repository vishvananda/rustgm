#![cfg_attr(guest_probe, no_std)]
#![cfg_attr(guest_probe, no_main)]

use core::ops::Deref;

struct Wrapper([u8; 2]);

impl Deref for Wrapper {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Wrapper {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

fn same(left: &Option<Wrapper>, right: Option<&Wrapper>) -> bool {
    left.as_deref() == right.map(AsRef::as_ref)
}

fn experiment() -> i32 {
    let left = Some(Wrapper([7, 9]));
    let right = Wrapper([7, 9]);
    let other = Wrapper([2, 3]);
    if same(&left, Some(&right)) && !same(&left, Some(&other)) {
        0
    } else {
        1
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
