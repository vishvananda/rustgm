#[derive(Copy,Clone)]#[repr(C)]enum E{A{x:u8,y:u32},B(bool)}union U{e:E,b:[u8;12]}const N:u32=unsafe{match (U{b:[0,0,0,0,3,0,0,0,34,0,0,0]}).e{E::A{x,y}=>x as u32+y,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
