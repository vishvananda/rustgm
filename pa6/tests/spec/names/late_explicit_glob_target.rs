pub use a::*;mod a{pub use crate::b::*;pub use crate::d::X;}mod b{pub struct X;}mod d{pub use crate::e::X;}mod e{pub struct X;}
