#![no_std]
// The injected `core` is a nameable crate-root item only in Rust 2015.
pub fn named() -> u32 {
    crate::core::base()
}
