#![allow(arithmetic_overflow)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->&'static u8{&(255u8+1)}fn main(){let _=f();}
