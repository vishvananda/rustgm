#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u8={let r:&[u8]=b"abc";let p=r as *const [u8];unsafe{(*p)[1]}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u8{let r:&[u8]=b"abc";let p=r as *const [u8];unsafe{(*p)[1]}}
fn main(){check(X==98u8);check(operate()==98u8);}
