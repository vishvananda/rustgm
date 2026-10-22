#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u32;3]=[3,5,7];const fn get(p:*const [u32],index:usize)->u32{unsafe{(*p)[index]}}const X:u32={let r:&[u32]=&A;get(r as *const [u32],2)};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u32{let r:&[u32]=&A;get(r as *const [u32],2)}
fn main(){check(X==7u32);check(operate()==7u32);}
