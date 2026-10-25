#![allow(dead_code,unused_variables)]
enum Never{}struct S<T>{value:T}fn probe(value:S<Never>)->u8{match value{}}
fn main(){}
