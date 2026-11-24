#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items)]
#![no_core]

extern crate base;

use base::{Held, call_lend, pick_early, shorten, unused, unused_generic, via_provider};

static STATIC_A: u8 = 11;
static STATIC_B: u8 = 3;

const CONST_SHORT: &'static u8 = Held { value: &STATIC_A }.shorter(&STATIC_B);

#[unsafe(no_mangle)]
pub extern "C" fn observe_early_lifetimes(out: &mut [u8; 11]) {
    let s = *shorten::<'static, 'static>(&STATIC_A, &STATIC_B);
    let c = *CONST_SHORT;
    let x = 7u8;
    let y = 5u8;
    let local = *shorten(&x, &y);
    let picked = *pick_early(&y, &STATIC_A);
    let u1 = unused(2);
    let u2 = unused_generic(4u8);
    let (p0, p1, p2, p3) = via_provider(&x, &y);
    let held = Held { value: &x };
    let lent = *call_lend(&held, &y);
    *out = [s, c, local, picked, u1, u2, p0, p1, p2, p3, lent];
}
