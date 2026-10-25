#![allow(dead_code,unused_imports)]
const fn divide(x:u8)->u8{1/x}
macro_rules! make {()=>{const _:u8=divide(0);}}
make!();fn main(){}
