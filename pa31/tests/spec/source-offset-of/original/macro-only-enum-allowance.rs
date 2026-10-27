#![feature(builtin_syntax,allow_internal_unstable)]
#[allow_internal_unstable(offset_of_enum)]macro_rules! m{()=>{builtin # offset_of(E,A.0)}}#[repr(u8)]enum E{A(u8)}const X:usize=m!();
