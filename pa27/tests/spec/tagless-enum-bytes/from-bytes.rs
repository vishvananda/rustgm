#[derive(Copy,Clone)]enum E{Only(u32)}union U{e:E,b:[u8;4],half:u16}const N:u32=unsafe{match(U{b:[37,0,0,0]}).e{E::Only(v)=>v}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
