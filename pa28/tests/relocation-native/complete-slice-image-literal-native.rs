#![allow(dead_code,unused_mut,invalid_reference_casting)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
union U{pointer:*const [u8],bytes:[u8;16],words:[usize;2]}static A:[u8;3]=[3,5,7];const X:U={let a:&[u8]=&A;let source=U{pointer:a};let mut target=U{bytes:[0;16]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],8);core::ptr::copy_nonoverlapping(&raw const source.bytes[8],&raw mut target.bytes[8],8);}target};
fn main(){check(unsafe{(*X.pointer)[1]}==5);}
