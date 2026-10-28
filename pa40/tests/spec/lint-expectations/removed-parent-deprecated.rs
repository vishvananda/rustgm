#![allow(dead_code,unused_variables)]
#![expect(deprecated)]
#[deprecated]fn old(){}#[test]fn gone(){old()}fn main(){}
