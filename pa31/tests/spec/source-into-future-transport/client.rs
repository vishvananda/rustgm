#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Immediate(u8);
impl base::Future for Immediate {
    type Output = u8;
}

struct Deferred(u8);
impl base::IntoFuture for Deferred {
    type Output = u8;
    type IntoFuture = Immediate;

    fn into_future(self) -> Immediate {
        Immediate(self.0)
    }
}

fn run<T: base::IntoFuture<IntoFuture = Immediate>>(value: T) -> u8 {
    value.into_future().0
}

#[unsafe(no_mangle)]
pub extern "C" fn observed() -> u8 {
    run(Deferred(41))
}
