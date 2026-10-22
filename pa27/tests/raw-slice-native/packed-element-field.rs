#![allow(dead_code,unused_mut,invalid_reference_casting)]
#[repr(C,packed)]struct P{tag:u8,value:u32}static A:[P;2]=[P{tag:0,value:3},P{tag:1,value:7}];const X:u32={let r:&[P]=&A;let p=r as *const [P];unsafe{(*p)[1].value}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u32{let r:&[P]=&A;let p=r as *const [P];unsafe{(*p)[1].value}}
fn main(){check(X==7u32);check(operate()==7u32);}
