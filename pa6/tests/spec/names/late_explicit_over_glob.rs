pub use a::X; mod a{pub use crate::b::*;pub use crate::c::*;pub use crate::d::X;}mod b{pub struct X;}mod c{pub struct X;}mod d{pub use crate::e::X;}mod e{pub struct X;}
