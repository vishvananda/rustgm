#![feature(builtin_syntax,core_intrinsics)]
#[repr(C)]struct S{a:u8,b:u32}
const X:usize=builtin # offset_of(S,(a));
