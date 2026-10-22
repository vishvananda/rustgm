#![allow(dead_code,unused_mut,invalid_reference_casting)]
#[repr(C,packed)]struct P{tag:u8,value:u32}static A:[P;2]=[P{tag:0,value:3},P{tag:1,value:7}];const X:u32={let r:&[P]=&A;let p=r as *const [P];unsafe{(*p)[1].value}};fn main(){let _=X;}
