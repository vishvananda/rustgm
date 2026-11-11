#![allow(dead_code)]
mod ops{pub use core::ops::Deref;}
struct Guard<'a,T>(&'a T);impl<'a,T>ops::Deref for Guard<'a,T>{type Target=T;fn deref(&self)->&T{self.0}}
fn check<'a,'b,T:ops::Deref<Target=u8>>(g:&'a Guard<'b,T>)->&'a u8{g.deref()}
