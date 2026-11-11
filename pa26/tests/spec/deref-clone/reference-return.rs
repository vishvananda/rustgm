#![allow(dead_code)]
use core::ops::Deref;
struct Guard<'a,T>(&'a T);impl<'a,T>Deref for Guard<'a,T>{type Target=T;fn deref(&self)->&T{self.0}}
fn check<'a,'b,T>(x:&'a Guard<'b,T>)->&'a Guard<'b,T>{x.clone()}
