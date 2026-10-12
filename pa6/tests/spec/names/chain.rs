use a::g as f;mod a{pub use crate::b::f as g;}mod b{pub fn f(){}}fn h(){f();}
