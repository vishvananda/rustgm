#![allow(dead_code,unused_mut,invalid_reference_casting)]
const X:u32={let a:[u32;0]=[];let r:&[u32]=&a;let p=r as *const [u32];unsafe{match *p{[]=>37,_=>0}}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u32{let a:[u32;0]=[];let r:&[u32]=&a;let p=r as *const [u32];unsafe{match *p{[]=>37,_=>0}}}
fn main(){check(X==37u32);check(operate()==37u32);}
