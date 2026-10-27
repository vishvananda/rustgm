#![feature(builtin_syntax)]
#[repr(C)]struct S{a:u8,b:u32}
const X:usize=builtin # r#offset_of(S,b);
