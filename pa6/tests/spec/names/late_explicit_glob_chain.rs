pub use a::*;mod a{pub use crate::b::*;pub use crate::d::*;}mod b{pub use crate::e::*;}mod d{pub use crate::e::*;}mod e{pub struct X;}
