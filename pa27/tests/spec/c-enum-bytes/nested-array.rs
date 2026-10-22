#[derive(Copy,Clone)]#[repr(C)]enum E{A([u32;2]),Empty}union U{e:E,b:[u8;12]}const N:u32=unsafe{match (U{e:E::A([17,20])}).e{E::A(v)=>v[0]+v[1],_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
