#![feature(builtin_syntax)]
struct D<T:?Sized>{a:u8,b:u32,tail:T}const X:usize=builtin # offset_of(D<dyn std::fmt::Debug>,b);
