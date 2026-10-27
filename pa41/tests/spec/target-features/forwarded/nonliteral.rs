#![allow(dead_code,unused_unsafe,internal_features)]
#![feature(x86_amx_intrinsics)]
macro_rules! make{($s:expr)=>{#[target_feature(enable=$s)]pub fn f(){}}}make!(3+4);
