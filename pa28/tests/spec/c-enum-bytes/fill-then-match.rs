#[derive(Copy,Clone)]#[repr(C)]enum E{A(u32),B(bool)}union U{e:E,b:[u8;8],tag:u32,half:u16}const N:u32=unsafe{let mut e=E::A(37);core::ptr::write_bytes(&raw mut e,0,1);match e{E::A(v)=>v,_=>1}};fn main(){check(N==0)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
