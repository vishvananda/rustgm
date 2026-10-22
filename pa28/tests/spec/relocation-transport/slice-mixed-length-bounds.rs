#![allow(dead_code,unused_mut,invalid_reference_casting)]
union U{pointer:*const [u8],bytes:[u8;16]}static A:[u8;3]=[3,5,7];static B:[u8;2]=[11,13];const X:u8={let a:&[u8]=&A;let source=U{pointer:a};let mut target=U{bytes:[0;16]};let b:&[u8]=&B;let other=U{pointer:b};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],8);core::ptr::copy_nonoverlapping(&raw const other.bytes[8],&raw mut target.bytes[8],8);(*target.pointer)[2]}};
fn main(){let _=X;}
