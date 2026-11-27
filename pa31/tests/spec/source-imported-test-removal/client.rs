#![feature(no_core, role_fixture)]
#![no_core]
extern crate base;

#[base::test]
fn test_body() {
    only_available_to_tests();
}

#[unsafe(no_mangle)]
pub extern "C" fn answer() -> u8 {
    43
}
