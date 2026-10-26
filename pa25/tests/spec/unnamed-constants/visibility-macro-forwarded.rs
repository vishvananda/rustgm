#![allow(dead_code,unused_imports)]
#![allow(unused_visibilities)]
macro_rules! twice {($i:item)=>{$i $i}}
twice!(#[warn(unused_visibilities)]pub const _:u8=1;);fn main(){}
