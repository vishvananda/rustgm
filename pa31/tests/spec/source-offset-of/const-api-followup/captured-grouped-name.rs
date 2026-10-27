#![feature(builtin_syntax,core_intrinsics)]
#[repr(C)]struct S{a:u8,b:u32}
macro_rules! m{($e:expr)=>{builtin # offset_of(S,$e)}}const X:usize=m!((a));
