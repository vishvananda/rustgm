mod glob {pub mod m {pub struct S;}}mod m {pub struct S;} fn f(){use glob::*;use crate::glob::m::S;}fn main(){}
