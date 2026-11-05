#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async fn first(){} pub async fn second(){}
fn same<T>(_:T,_:T){}
fn consumer(){same(first(),second());}
