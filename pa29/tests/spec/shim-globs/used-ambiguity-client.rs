#![no_std]
// Using the shim's ambiguous `shared` is an error.
pub fn shared() -> u32 {
    core::shared()
}
