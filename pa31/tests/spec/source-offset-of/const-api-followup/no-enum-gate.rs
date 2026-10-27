#![feature(builtin_syntax,core_intrinsics)]
#[repr(u8)]enum E{A(u8)}const X:usize=builtin # offset_of(E,A.0);
