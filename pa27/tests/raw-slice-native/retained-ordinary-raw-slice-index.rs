#![allow(dead_code,unused_mut)]
static A:[u8;3]=[3,5,7];const X:u8={let a:&[u8]=&A;let pointer=a as *const [u8];unsafe{(*pointer)[1]}};
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u8{let a:&[u8]=&A;let pointer=a as *const [u8];unsafe{(*pointer)[1]}}
fn main(){check(X==5u8);check(operate()==5u8);}
