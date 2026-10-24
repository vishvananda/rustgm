#![allow(dead_code,unused_variables,unused_attributes,internal_features)]
#![forbid(unused_must_use)]
#[must_use]fn f()->u32{1}#[warn(unused_must_use)]fn probe(){f();}
fn main(){}
