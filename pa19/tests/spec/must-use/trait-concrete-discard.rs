#![allow(dead_code,unused_variables,unused_attributes,internal_features)]
#![deny(unused_must_use)]
#[must_use]trait T{}struct S;impl T for S{}fn f()->S{S}fn probe(){f();}
fn main(){}
