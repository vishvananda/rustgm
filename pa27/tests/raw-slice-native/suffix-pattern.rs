#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u32;3]=[3,5,7];const X:u32={let r:&[u32]=&A;let p=r as *const [u32];unsafe{match *p{[first,..,last]=>first+last,_=>0}}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u32{let r:&[u32]=&A;let p=r as *const [u32];unsafe{match *p{[first,..,last]=>first+last,_=>0}}}
fn main(){check(X==10u32);check(operate()==10u32);}
