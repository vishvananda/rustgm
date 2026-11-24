#![feature(async_fn_traits, no_core, role_fixture, unboxed_closures)]
#![allow(internal_features)]
#![no_core]
extern crate base;

struct Immediate(u8);
impl base::Future for Immediate {
    type Output = u8;
}

struct Deferred(u8);
impl base::AsyncFnOnce<()> for Deferred {
    type CallOnceFuture = Immediate;
    type Output = u8;

    extern "rust-call" fn async_call_once(self, _: ()) -> Immediate {
        Immediate(self.0)
    }
}

struct Borrowed<'a>(&'a u8);
impl base::Future for Borrowed<'_> {
    type Output = u8;
}
impl base::AsyncFnMut<()> for Deferred {
    type CallRefFuture<'a> = Borrowed<'a> where Self: 'a;

    extern "rust-call" fn async_call_mut(&mut self, _: ()) -> Borrowed<'_> {
        Borrowed(&self.0)
    }
}

fn run<T: base::AsyncFnOnce<(), CallOnceFuture = Immediate>>(value: T) -> u8 {
    value.async_call_once(()).0
}

#[unsafe(no_mangle)]
pub extern "C" fn observed(which: u8) -> u8 {
    match which {
        0 => {
            let mut instance = Deferred(41);
            *base::AsyncFnMut::async_call_mut(&mut instance, ()).0
        }
        _ => run(Deferred(41)),
    }
}
