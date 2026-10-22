#![allow(dead_code,unused_mut,invalid_reference_casting)]
#[repr(C,packed)]struct P{tag:u8,values:[u32;2]}static A:P=P{tag:0,values:[3,7]};union U{p:*const [u32],parts:(*const u32,usize)}const X:u32={let u=U{parts:(&raw const A.values[0],2)};unsafe{core::ptr::read_unaligned(&raw const (*u.p)[1])}};fn main(){let _=X;}
