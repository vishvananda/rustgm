#![allow(dead_code,unused_imports,incomplete_features,internal_features)]
#![feature(trait_alias)]
trait A{fn a(&self)->u8;}impl A for u8{fn a(&self)->u8{7}}
trait Alias=A;fn use_dyn(value:&dyn Alias)->u8{value.a()}
fn main(){let _=use_dyn(&0_u8);}
