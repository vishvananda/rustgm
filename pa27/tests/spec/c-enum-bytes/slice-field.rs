static S:[u8;2]=[17,20];#[derive(Copy,Clone)]#[repr(C)]enum E{A(&'static [u8]),Empty}union U{e:E,b:[u8;24]}const N:u8=unsafe{match (U{e:E::A(&S)}).e{E::A(v)=>v[0]+v[1],_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
