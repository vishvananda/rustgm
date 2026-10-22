#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let mut a=[3u32,5,7];let r:&mut [u32]=&mut a;let p=r as *mut [u32];unsafe{(*p)[1]=41;(*p)[1]}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u32{let mut a=[3u32,5,7];let r:&mut [u32]=&mut a;let p=r as *mut [u32];unsafe{(*p)[1]=41;(*p)[1]}}
fn main(){check(X==41u32);check(operate()==41u32);}
