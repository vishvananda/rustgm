#![feature(no_core)]
#![no_core]
extern crate gates;
// The imported row's allowance list admits `cfg(ub_checks)` in its expansion.
pub fn checked() -> bool {
    gates::checked!()
}
