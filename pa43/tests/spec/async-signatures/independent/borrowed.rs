#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async fn value<'a>(x:&'a u8)->&'a u8{x}
fn require<'a,F:Future<Output=&'a u8>>(_:F){}
fn consumer<'a>(x:&'a u8){require(value(x));}
