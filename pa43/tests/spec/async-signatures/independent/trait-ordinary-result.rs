#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub trait Async { fn compute(&self)->u8; }
struct Thing;
impl Async for Thing { async fn compute(&self)->u8{7u8} }
