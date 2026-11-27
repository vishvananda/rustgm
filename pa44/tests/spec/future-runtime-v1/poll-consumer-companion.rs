use core::{future::Future,pin::Pin,task::{Context,Poll}};
pub async fn value()->u8{7}
pub async fn driven<T:Copy>(_:T,cx:&mut Context<'_>)->u8{let mut f=value();let pin=unsafe{Pin::new_unchecked(&mut f)};match Future::poll(pin,cx){Poll::Ready(x)=>x,Poll::Pending=>0}}
