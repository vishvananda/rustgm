#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async fn value()->impl Copy{7u8}
fn require<F:Future>(_:F)where F::Output:Copy{}
fn consumer(){require(value());}
