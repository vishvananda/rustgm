#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async unsafe fn cleanup<T:?Sized>(_slot:&mut T){}
fn require<F:Future<Output=()>>(_:F){}
fn consumer(slot:&mut u8){let constructor=cleanup::<u8>;require(unsafe{constructor(slot)});}
