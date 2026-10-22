#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C,packed)]struct P{a:u8,b:u32}
const X:u32={let mut p=P{a:0,b:0};unsafe{core::ptr::write_bytes(&raw mut p,1,1);}p.b};fn main(){check(X==0x01010101);let mut p=P{a:0,b:0};unsafe{core::ptr::write_bytes(&raw mut p,7,1);}check(p.a==7);check(p.b==0x07070707);}
