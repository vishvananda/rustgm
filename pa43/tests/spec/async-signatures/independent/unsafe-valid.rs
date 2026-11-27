#![allow(dead_code,unused_variables,async_fn_in_trait)]
use core::future::Future;
pub async unsafe fn value(){}
fn consumer(){unsafe{value();}}
