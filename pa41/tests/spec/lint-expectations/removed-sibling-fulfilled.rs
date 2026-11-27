#![allow(dead_code,unused_variables)]
#[test]fn gone(){missing!();}#[expect(unreachable_patterns)]fn live(){match true{_=>(),false=>()}}fn main(){}
