#![allow(dead_code,unused_imports)]
const fn divide(x:u8)->u8{1/x}
fn never_called(){const _:u8=divide(0);}fn main(){}
