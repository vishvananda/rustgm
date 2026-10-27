#![feature(builtin_syntax)]
struct D<T:?Sized>{a:u8,tail:T}const X:usize=builtin # offset_of(D<[u8]>,tail);
