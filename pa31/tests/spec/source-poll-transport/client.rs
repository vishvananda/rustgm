#![feature(coroutine_trait)]
#![feature(no_core, role_fixture)]
#![allow(internal_features)]
#![no_core]
extern crate base;

fn inspect(value: base::Poll<base::CoroutineState<u8, u8>>) -> u8 {
    match value {
        base::Poll::Pending => 1,
        base::Poll::Ready(base::CoroutineState::Yielded(n)) => n,
        base::Poll::Ready(base::CoroutineState::Complete(n)) => n,
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn observed(which: u8) -> u8 {
    match which {
        0 => inspect(base::Poll::Pending),
        1 => inspect(base::Poll::Ready(base::CoroutineState::Yielded(7))),
        _ => inspect(base::Poll::Ready(base::CoroutineState::Complete(41))),
    }
}
