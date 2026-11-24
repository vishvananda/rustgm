#![feature(no_core, transmute_fixture, transmutability)]
#![allow(internal_features)]
#![no_core]
extern crate base;

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    let options = base::Assume { alignment: true, lifetimes: false, safety: true, validity: false };
    if options.lifetimes || options.validity {
        13
    } else if options.alignment && options.safety {
        41
    } else {
        13
    }
}
