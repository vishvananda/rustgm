#![feature(builtin_syntax,offset_of_enum,core_intrinsics)]
#[repr(u8)]enum E{A(u8)}const X:usize=builtin # offset_of(E,A.0);
