#![feature(builtin_syntax,core_intrinsics)]
macro_rules! m{($e:expr)=>{builtin # offset_of((u8,u32),$e)}}const X:usize=m!(0);
