#![no_std]
// The shim's glob carries no `pub(crate)` name of core.
pub fn hidden() -> u32 {
    core::internal()
}
