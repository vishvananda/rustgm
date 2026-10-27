#![allow(dead_code,unused_unsafe,internal_features)]
#![feature(x86_amx_intrinsics)]
macro_rules! make{($s:literal)=>{#[target_feature(enable=$s)]pub fn f(){}}}make!("amx-tile");
