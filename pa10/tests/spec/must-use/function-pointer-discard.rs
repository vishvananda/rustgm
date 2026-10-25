#![allow(dead_code,unused_variables,unused_attributes,internal_features)]
#![deny(unused_must_use)]
#[must_use]fn f()->u32{1}fn probe(){let pointer:fn()->u32=f;pointer();}
fn main(){}
