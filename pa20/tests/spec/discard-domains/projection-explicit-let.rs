#![allow(dead_code,unused_variables,unreachable_code)]
#[must_use="BOUND"]trait Mark{}trait Has{type Out:Mark;fn make()->Self::Out;}fn probe<T:Has>(){let _=T::make();}
fn main(){}
