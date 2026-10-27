#![feature(builtin_syntax)]
macro_rules! m{($l:literal)=>{builtin # offset_of((u8,u32),$l)}}const X:usize=m!(0);
