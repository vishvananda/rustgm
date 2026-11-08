#![no_std]
// The shim's own `option` module shadows the glob's.
fn none() -> core::option::Option<u32> {
    core::option::Option::None
}
pub fn zero() -> u32 {
    match none() {
        core::option::Option::Some(value) => value,
        core::option::Option::None => 0,
    }
}
