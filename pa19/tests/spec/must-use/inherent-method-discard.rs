#![allow(dead_code,unused_variables,unused_attributes,internal_features)]
#![deny(unused_must_use)]
struct S;impl S{#[must_use]fn f(&self)->u32{1}}fn probe(){S.f();}
fn main(){}
