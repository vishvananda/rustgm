#![allow(dead_code,unused_mut)]
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait T{fn value(&self)->u32;}impl T for u32{fn value(&self)->u32{*self}}impl T for u64{fn value(&self)->u32{*self as u32}}union U{pointer:*const dyn T,bytes:[u8;16],words:[usize;2]}static A:u32=37;static B:u32=91;static C:u64=125;const X:*const dyn T={let source={let typed:&dyn T=&A;U{pointer:typed}};let other={let typed:&dyn T=&B;U{pointer:typed}};let mut target=U{bytes:[0;16]};unsafe{core::ptr::copy_nonoverlapping(&raw const source.bytes[0],&raw mut target.bytes[0],12);core::ptr::copy_nonoverlapping(&raw const other.bytes[12],&raw mut target.bytes[12],4);target.pointer}};
fn main(){check(unsafe{(*X).value()}==37);}
