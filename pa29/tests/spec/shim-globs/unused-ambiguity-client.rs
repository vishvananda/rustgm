#![no_std]
// The shim's ambiguous `shared` is never used, so it is no error; the
// modules it came from still resolve.
pub fn left() -> u32 {
    core::only_left()
}
pub fn right() -> u32 {
    core::right::shared()
}
