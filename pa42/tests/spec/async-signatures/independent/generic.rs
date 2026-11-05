#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async fn value<T>(x:T)->T{x}
fn require<T,F:Future<Output=T>>(_:F){}
fn consumer<T>(x:T){require(value(x));}
