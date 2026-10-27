#![feature(builtin_syntax,core_intrinsics)]
type T=(u8,(u8,u8,u32));const X:usize=builtin # offset_of(T,1.2);
