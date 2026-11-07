#![feature(no_core)]
#![no_core]
extern crate panics;
// The second rule names the provider's unstable `panicking::formatted`,
// which `panic_2021`'s own allowance list admits.
pub fn formatted() -> u32 {
    panics::panic!("code {}", 7)
}
