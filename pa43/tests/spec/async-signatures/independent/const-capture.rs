#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async fn value<const N:usize>(){}
fn same<T>(_:T,_:T){}
fn consumer(){same(value::<3>(),value::<3>());}
