#![feature(builtin_syntax)]
#[repr(u8)]enum E{A(u8)}#[cfg(any())]fn f(){let _=builtin # offset_of(E,A.0);}
