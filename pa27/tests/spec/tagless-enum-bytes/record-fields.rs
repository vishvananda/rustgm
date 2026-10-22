#[derive(Copy,Clone)]enum E{Only{a:u8,b:u32}}union U{e:E,b:[u8;8]}const N:u32=unsafe{match(U{e:E::Only{a:3,b:34}}).e{E::Only{a,b}=>a as u32+b}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
