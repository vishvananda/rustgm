#![feature(no_core, location_fixture)]
#![no_core]

use base::Clone;

#[unsafe(no_mangle)]
pub extern "C" fn observe(value: u8) -> u8 {
    let wrapped = base::ManuallyDrop {
        value: base::MaybeDangling(value),
    };
    wrapped.clone().value.0
}
