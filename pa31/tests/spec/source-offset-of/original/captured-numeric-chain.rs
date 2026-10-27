#![feature(builtin_syntax)]
macro_rules! m{($e:expr)=>{builtin # offset_of((u8,(u8,u8,u32)),$e)}}const X:usize=m!(1.2);
