#![feature(no_core)]
#![no_core]
extern crate panics;
// The expansion is `if !flag { $crate::panic::panic_2021!(..) }`; the
// provider declares no `Not` language item.
pub fn checked(flag: bool) -> u32 {
    panics::assert!(flag, "flag {}", 1);
    7
}
