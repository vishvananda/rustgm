#![feature(async_gen_internals)]
#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Immediate(u8);
impl base::Future for Immediate {
    type Output = u8;
}

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    let ready = base::Poll::async_gen_ready(41u8);
    let stalled = <base::Poll<base::Option<u8>>>::PENDING;
    let done = <base::Poll<base::Option<u8>>>::FINISHED;
    match (ready, stalled, done) {
        (base::Poll::Ready(base::Option::Some(n)), base::Poll::Pending,
            base::Poll::Ready(base::Option::None)) => n,
        _ => 0,
    }
}
