#[derive(Copy,Clone)]enum E{A(u32),Empty}#[derive(Copy,Clone)]enum F{A(E),B{a:u8,b:E},Empty}union U{e:F,b:[u8;64]}const N:u32=unsafe{match(U{e:F::B{a:17,b:E::A(20)}}).e{F::B{a,b:E::A(b)}=>a as u32+b,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
