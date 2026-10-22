#[derive(Copy,Clone)]enum E{A{a:u8,b:u32},Empty}union U{e:E,b:[u8;32]}const N:u32=unsafe{match(U{e:E::A{a:17,b:20}}).e{E::A{a,b}=>a as u32+b,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
