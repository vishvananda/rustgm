use std::future::Future;
use std::pin::Pin;
use std::task::{Context,Poll};
fn context()->Context<'static>{Context::from_waker(std::task::Waker::noop())}
pub async fn value()->u8{3}
#[unsafe(no_mangle)]pub extern "C" fn observe()->u8{
    let mut future=value();
    let mut cx=context();
    let _=Future::poll(unsafe{Pin::new_unchecked(&mut future)},&mut cx);
    match Future::poll(unsafe{Pin::new_unchecked(&mut future)},&mut cx){
        Poll::Ready(value)=>value,
        Poll::Pending=>loop{},
    }
}
fn main(){}
