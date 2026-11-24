#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn shorten<'a>(cx: base::Context<'static>) -> base::Context<'a> {
    cx
}

#[unsafe(no_mangle)]
pub extern "C" fn reject_shorten() -> u8 {
    let _ = shorten as fn(base::Context<'static>) -> base::Context<'_>;
    0
}
