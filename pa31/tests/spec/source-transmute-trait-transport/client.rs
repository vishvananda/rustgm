#![feature(no_core, source_operator_fixture, transmutability)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    let options = base::Assume { alignment: true, lifetimes: false, safety: true, validity: false };
    if options.alignment && options.safety {
        41
    } else {
        13
    }
}
