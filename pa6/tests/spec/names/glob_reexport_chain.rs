mod a{pub struct S;}mod b{pub use crate::a::*;}use b::*;fn f(){S;}
