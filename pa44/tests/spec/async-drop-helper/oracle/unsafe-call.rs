#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async unsafe fn cleanup<T:?Sized>(_slot:&mut T){}
fn consumer<T:?Sized>(slot:&mut T){let _pending=cleanup(slot);}
