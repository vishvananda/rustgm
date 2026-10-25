#![allow(dead_code,unused_imports)]
#![deny(unused_visibilities)]
macro_rules! make {()=>{#[allow(unused_visibilities)]pub const _:u8=1;}}
make!();fn main(){}
