#![no_std]
// A `no_std` crate has no `std` in its extern prelude.
pub fn named() -> u32 {
    std::base()
}
