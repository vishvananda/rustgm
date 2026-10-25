#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(rustc_attrs,allow_internal_unstable)]
trait A{}
#[allow_internal_unstable(trait_alias)]macro_rules! make{($($item:tt)*)=>{$($item)*}}
make!(trait Alias=A;);fn main(){}
