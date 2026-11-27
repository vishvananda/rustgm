#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async fn value<T>(){}
fn same<T>(_:T,_:T){}
fn consumer(){same(value::<u8>(),value::<u16>());}
