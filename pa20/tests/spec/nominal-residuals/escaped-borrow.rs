#![allow(dead_code,unused_variables)]
pub fn probe<T:'static>(input:Option<u8>,output:&T)->Option<&'static T>{input?;Some(output)}
fn main(){}
