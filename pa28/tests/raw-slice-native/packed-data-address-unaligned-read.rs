#![allow(dead_code,unused_mut,invalid_reference_casting)]
#[repr(C,packed)]struct P{tag:u8,values:[u32;2]}static A:P=P{tag:0,values:[3,7]};union U{p:*const [u32],parts:(*const u32,usize)}const X:u32={let u=U{parts:(&raw const A.values[0],2)};unsafe{core::ptr::read_unaligned(&raw const (*u.p)[1])}};fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn operate()->u32{let u=U{parts:(&raw const A.values[0],2)};unsafe{core::ptr::read_unaligned(&raw const (*u.p)[1])}}
fn main(){check(X==7u32);check(operate()==7u32);}
