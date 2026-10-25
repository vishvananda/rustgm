#![allow(dead_code,unused_variables,unused_attributes,internal_features)]
#![deny(unused_must_use)]
#[must_use="retain type"]enum S{A}fn probe(){S::A;}
fn main(){}
