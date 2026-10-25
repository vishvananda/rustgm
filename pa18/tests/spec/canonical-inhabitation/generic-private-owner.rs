#![allow(dead_code,unused_variables)]
mod owner{enum Never{}struct S<T>{value:T}fn probe(value:S<Never>)->u8{match value{}}}
fn main(){}
