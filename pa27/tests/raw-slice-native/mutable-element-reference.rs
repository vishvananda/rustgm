#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let mut a=[3u32,5];let r:&mut [u32]=&mut a;let p=r as *mut [u32];let q=unsafe{&mut (*p)[1]};*q=41;*q};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u32{let mut a=[3u32,5];let r:&mut [u32]=&mut a;let p=r as *mut [u32];let q=unsafe{&mut (*p)[1]};*q=41;*q}
fn main(){check(X==41u32);check(operate()==41u32);}
