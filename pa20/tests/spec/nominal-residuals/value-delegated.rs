#![allow(dead_code,unused_variables)]
pub fn make<T>(output:T)->Option<T>{Some(output)}
pub fn probe<T>(input:Option<u8>,output:T)->Option<T>{input?;make(output)}
fn main(){}
