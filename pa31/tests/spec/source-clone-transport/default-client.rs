#![feature(no_core, const_trait_impl, const_clone, clone_fixture, const_destruct)]
#![no_core]

extern crate base;

struct Local(u8);

const impl base::Clone for Local {
    fn clone(&self) -> Self { Local(self.0) }
}

fn replace(value: u8) -> u8 {
    let mut previous = Local(41u8);
    let incoming = Local(value);
    base::Clone::clone_from(&mut previous, &incoming);
    previous.0
}

#[unsafe(no_mangle)]
pub extern "C" fn observed(value: u8) -> u8 { replace(value) }
