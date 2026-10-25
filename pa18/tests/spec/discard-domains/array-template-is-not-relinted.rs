#![allow(dead_code,unused_variables,unreachable_code)]
#[must_use]struct S;fn probe<const N:usize>(value:[S;N]){value;}fn reached(){probe::<1>([S]);}
fn main(){}
