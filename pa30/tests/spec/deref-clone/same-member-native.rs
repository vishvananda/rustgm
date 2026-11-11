#![allow(dead_code)]
mod ops{pub use core::ops::Deref;}
unsafe extern "C"{fn mark(step:u8);}
struct Value(u8);impl ops::Deref for Value{type Target=u8;fn deref(&self)->&u8{unsafe{mark(2)};&self.0}}
struct Guard<'a,T>(&'a T);impl<'a,T>ops::Deref for Guard<'a,T>{type Target=T;fn deref(&self)->&T{unsafe{mark(1)};self.0}}
fn check<'a,'b,T:ops::Deref<Target=u8>>(g:&'a Guard<'b,T>)->&'a u8{g.deref()}
#[unsafe(no_mangle)]pub extern "C" fn code()->u8{let value=Value(23u8);let guard=Guard(&value);*check(&guard)}
