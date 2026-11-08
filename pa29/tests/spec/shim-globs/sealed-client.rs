#![no_std]
// The shim's glob carries no private module of core.
pub fn hidden() -> u32 {
    core::sealed::hidden()
}
