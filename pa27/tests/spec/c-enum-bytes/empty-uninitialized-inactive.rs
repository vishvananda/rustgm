#[derive(Copy,Clone)]#[repr(C)]enum E{A(u32),Empty}union U{e:E,tag:u32}const N:bool=unsafe{match (U{tag:1}).e{E::Empty=>true,_=>false}};fn main(){check(N)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
