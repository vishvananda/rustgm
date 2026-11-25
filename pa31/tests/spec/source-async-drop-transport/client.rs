#![no_std]
#![allow(internal_features)]
#![feature(no_core, lang_items)]
#![no_core]

extern crate base;

use base::{AsyncDrop, AsyncWork, Context, Future, Pin, Poll, ReadyWork, Sized};

struct Done(u32);

impl Future for Done {
    type Output = u32;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<u32> {
        Poll::Ready(self.pointer.0)
    }
}

fn _use_async_drop_bound<T: AsyncDrop + ?Sized>(x: &T) -> u32 {
    base::accept_async_drop(x)
}

fn _use_async_work_bound<T: AsyncWork>(x: &T) -> u32 {
    base::accept_async_work(x)
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_ready_work() -> u32 {
    base::check_value(ReadyWork(42))
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_poll_tag() -> u32 {
    let mut done = Done(17);
    let pinned = Pin { pointer: &mut done };
    let unit = ();
    let mut cx = Context { _marker: &unit };
    match Future::poll(pinned, &mut cx) {
        Poll::Ready(value) => value,
        Poll::Pending => 0,
    }
}
