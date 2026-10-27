#![feature(builtin_syntax)]
type T=(u8,(u8,u8,u32));const X:usize=builtin # offset_of(T,1.2);
