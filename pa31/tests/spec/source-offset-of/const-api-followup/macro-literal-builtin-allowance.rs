#![feature(allow_internal_unstable,core_intrinsics)]
#[allow_internal_unstable(builtin_syntax)]macro_rules! m{()=>{builtin # offset_of(S,b)}}#[repr(C)]struct S{a:u8,b:u32}
const X:usize=m!();
