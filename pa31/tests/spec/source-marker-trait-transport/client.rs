#![feature(no_core)]
#![no_core]
extern crate base;

struct Local(u8);

impl base::Clone for Local {
    fn clone(&self) -> Self {
        Local(self.0)
    }
}

unsafe impl base::CloneFromCell for Local {}

#[unsafe(no_mangle)]
pub extern "C" fn observed_provider() -> u8 {
    base::observe_marker(41, &0u8)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_local(value: u8) -> u8 {
    base::observe_marker(value, &Local(0u8))
}
