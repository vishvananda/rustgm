#[derive(Copy,Clone)]#[repr(C)]enum E{A=3,B=7}union U{e:E,tag:u32}const N:u8=unsafe{match (U{tag:7}).e{E::B=>37,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
