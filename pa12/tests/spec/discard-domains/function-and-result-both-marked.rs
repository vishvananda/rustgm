#![allow(dead_code,unused_variables)]
#[must_use="TYPE"]struct S;#[must_use="FUNCTION"]fn f()->S{S}fn probe(){f();}
fn main(){}
