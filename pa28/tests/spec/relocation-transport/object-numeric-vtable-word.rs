#![allow(dead_code,unused_mut)]
trait T{fn value(&self)->u32;}impl T for u32{fn value(&self)->u32{*self}}impl T for u64{fn value(&self)->u32{*self as u32}}union U{pointer:*const dyn T,bytes:[u8;16],words:[usize;2]}static A:u32=37;static B:u32=91;static C:u64=125;const X:usize={let source=U{pointer:&A as &dyn T};unsafe{source.words[1]}};
fn main(){let _=X;}
