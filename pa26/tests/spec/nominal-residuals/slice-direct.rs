#![allow(dead_code,unused_variables)]
pub fn probe<T>(input:Option<u8>,output:&[T])->Option<&[T]>{input?;Some(output)}
fn main(){}
