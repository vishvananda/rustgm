#![allow(dead_code,unused_unsafe,internal_features)]
#![feature(rustc_attrs,allow_internal_unstable)]
#[allow_internal_unstable(x86_amx_intrinsics)]macro_rules! make{($a:meta)=>{#[$a]pub fn f(){}}}make!(target_feature(enable="amx-tile"));
