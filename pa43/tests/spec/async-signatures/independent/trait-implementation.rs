#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub trait Async { async fn compute(&self)->u8; }
struct Thing;
impl Async for Thing { async fn compute(&self)->u8{7u8} }
fn require<F:Future<Output=u8>>(_:F){}
fn consumer<T:Async>(x:&T){require(x.compute());}
