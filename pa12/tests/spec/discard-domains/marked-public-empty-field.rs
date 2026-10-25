#![allow(dead_code,unused_variables,unreachable_code)]
pub enum Never{}#[must_use]pub struct S{pub never:Never}fn probe(value:S){value;}
fn main(){}
