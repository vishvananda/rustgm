#[derive(Copy,Clone)]#[repr(C)]enum E{A(u32),B(bool)}union U{e:E,b:[u8;8],tag:u32,half:u16}const N:bool=unsafe{match (U{b:[1,0,0,0,1,0,0,0]}).e{E::B(v)=>v,_=>false}};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
