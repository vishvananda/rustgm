#![feature(builtin_syntax)]
mod m{pub struct S{a:u8}}const X:usize=builtin # offset_of(m::S,a);
