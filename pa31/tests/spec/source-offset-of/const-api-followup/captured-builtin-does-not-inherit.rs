#![feature(allow_internal_unstable,core_intrinsics)]
#[allow_internal_unstable(builtin_syntax)]macro_rules! m{($e:expr)=>{$e}}#[repr(C)]struct S{a:u8,b:u32}
const X:usize=m!(builtin # offset_of(S,b));
