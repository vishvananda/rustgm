#![allow(dead_code,unused_variables)]
mod owner{enum Never{}struct S{value:Never}fn probe(value:S)->u8{match value{}}}
fn main(){}
