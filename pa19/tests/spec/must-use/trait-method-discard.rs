#![allow(dead_code,unused_variables,unused_attributes,internal_features)]
#![deny(unused_must_use)]
trait T{#[must_use]fn f(&self)->u32;}struct S;impl T for S{fn f(&self)->u32{1}}fn probe(){S.f();}
fn main(){}
