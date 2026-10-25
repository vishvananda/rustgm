#![allow(dead_code,unused_variables,internal_features)]
#[deny(duplicate_macro_attributes)]mod m{#[test]#[test]fn gone(){missing!();}}
fn main(){}
