#![allow(dead_code,unused_unsafe,internal_features)]
macro_rules! make{($s:literal)=>{#[doc=$s]pub struct S;#[cfg(target_arch=$s)]pub fn f(){}}}make!("x86_64");
