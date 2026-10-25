#![allow(dead_code,unused_variables)]
enum Never{}mod owner{pub struct S<T>{value:T}}fn probe(value:owner::S<Never>)->u8{match value{}}
fn main(){}
