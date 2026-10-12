mod a{pub struct S;pub use crate::b::*;}mod b{pub use crate::a::*;}fn f(){b::S;}
