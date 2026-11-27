#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async fn value()->u8{7u8}
fn require<F:Future<Output=u16>>(_:F){}
fn consumer(){require(value());}
