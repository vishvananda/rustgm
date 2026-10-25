#![allow(dead_code,unused_variables,unreachable_code)]
enum Never{}#[must_use]struct S<T>{value:T}fn probe(value:S<Never>){value;}
fn main(){}
