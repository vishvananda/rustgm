mod m{pub struct S;}mod glob{pub(crate) use crate::m;}fn f(){use glob::*;use m::S;}fn main(){}
