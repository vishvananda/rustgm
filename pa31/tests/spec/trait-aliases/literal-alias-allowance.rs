#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(rustc_attrs,allow_internal_unstable)]
trait A{}
#[allow_internal_unstable(trait_alias)]macro_rules! make{()=>{trait Alias=A;}}
make!();fn main(){}
