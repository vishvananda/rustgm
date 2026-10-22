#[derive(Copy,Clone)]#[repr(C)]enum E{A=-2147483649,B=7}union U{e:E,tag:i64}const N:u8=unsafe{match (U{tag:-2147483649}).e{E::A=>37,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
