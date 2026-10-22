#[derive(Copy,Clone)]#[repr(C)]enum E{A=4294967296,B=4294967297}union U{e:E,tag:u64}const N:u8=unsafe{match (U{tag:4294967297}).e{E::B=>37,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
