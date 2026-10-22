#![allow(dead_code,unused_mut,invalid_reference_casting)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait T{fn value(&self)->u32;}impl T for u32{fn value(&self)->u32{*self}}union U{pointer:*const dyn T,bytes:[u8;16]}static A:u32=37;const X:*const dyn T={let source={let typed:&dyn T=&A;U{pointer:typed}};let mut target=U{bytes:[0;16]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],8);core::ptr::copy_nonoverlapping(&raw const source.bytes[8],&raw mut target.bytes[8],8);target.pointer}};
fn main(){check(unsafe{(*X).value()}==37);}
