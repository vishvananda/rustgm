#![feature(builtin_syntax)]
#[repr(u8)]enum E{A(u8)}const X:usize=builtin # offset_of(E,A.0);
