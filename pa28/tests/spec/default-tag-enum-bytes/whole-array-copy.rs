#[derive(Copy,Clone)]enum E{A(u32),Empty}const N:u32=unsafe{let a=[E::A(17),E::A(20)];let mut b=[E::Empty;2];core::ptr::copy_nonoverlapping(&raw const a,&raw mut b,1);match b{[E::A(a),E::A(b)]=>a+b,_=>0}};fn main(){check(N==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
