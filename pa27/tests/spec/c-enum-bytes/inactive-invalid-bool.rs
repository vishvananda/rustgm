#[derive(Copy,Clone)]#[repr(C)]enum E{A(u32),B(bool)}union U{e:E,b:[u8;8],tag:u32,half:u16}const N:u32=unsafe{match (U{b:[0,0,0,0,37,0,0,0]}).e{E::A(v)=>v,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
