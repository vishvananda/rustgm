#![allow(dead_code,unused_mut,invalid_reference_casting)]
static A:[u32;3]=[3,5,7];union U{reference:&'static [u32],bytes:[u8;16]}const X:u32={let source=U{reference:&A};let mut target=U{bytes:[0;16]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],16);target.reference[1]}};fn main(){let _=X;}
