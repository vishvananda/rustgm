#![allow(dead_code,unused_variables,unused_attributes,internal_features)]
#![warn(unused_must_use)]
#[must_use="retain result"]fn f()->u32{1}fn probe(){f();}
fn main(){}
