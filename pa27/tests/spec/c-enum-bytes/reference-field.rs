static S:u32=37;#[derive(Copy,Clone)]#[repr(C)]enum E{A(&'static u32),Empty}union U{e:E,b:[u8;16]}const N:u32=unsafe{match (U{e:E::A(&S)}).e{E::A(v)=>*v,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
