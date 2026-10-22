#![allow(dead_code,unused_mut)]
static A:[u8;3]=[3,5,7];union U{pointer:*const [u8],bytes:[u8;16]}const X:*const [u8]={let [ref prefix @ ..,_]=A;let a:&[u8]=&A;let b:&[u8]=prefix;let source=U{pointer:a};let other=U{pointer:b};let mut target=U{bytes:[0;16]};unsafe{core::ptr::copy_nonoverlapping(&raw const other.bytes[4],&raw mut target.bytes[4],4);core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],4);core::ptr::copy_nonoverlapping(&raw const other.bytes[8],&raw mut target.bytes[8],8);target.pointer}};
fn main(){let _=X;}
