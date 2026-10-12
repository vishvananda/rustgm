mod glob {pub mod m {pub struct S;}}mod m {pub struct S;} fn f(){use glob::*;let _=m::S;}fn main(){}
