#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u8;3]=[3,5,7];union U{p:*const [u8],words:[usize;2]}const X:u8={let r:&[u8]=&A;let mut u=U{p:r};unsafe{u.words[1]=5;(*u.p)[1]}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u8{let r:&[u8]=&A;let mut u=U{p:r};unsafe{u.words[1]=5;(*u.p)[1]}}
fn main(){check(X==5u8);check(operate()==5u8);}
