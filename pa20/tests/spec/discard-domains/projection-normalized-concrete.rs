#![allow(dead_code,unused_variables,unreachable_code)]
#[must_use="BOUND"]trait Mark{}trait Has{type Out:Mark;fn make()->Self::Out;}struct S;impl Mark for S{}impl Has for S{type Out=S;fn make()->S{S}}fn probe(){S::make();}
fn main(){}
