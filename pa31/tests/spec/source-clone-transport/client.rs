#![feature(no_core, const_trait_impl, const_clone, clone_fixture, trivial_clone)]
#![no_core]

extern crate base;

struct Local(u8);

const impl base::Clone for Local {
    fn clone(&self) -> Self { Local(self.0) }
}

const unsafe impl base::TrivialClone for Local {}

fn duplicate<T: base::TrivialClone>(value: &T) -> T {
    base::Clone::clone(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed(value: u8) -> u8 {
    duplicate(&Local(value)).0
}
