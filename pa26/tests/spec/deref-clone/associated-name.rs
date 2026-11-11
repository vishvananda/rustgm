#![allow(dead_code)]
use core::ops::Deref;
struct Guard<'a,T>(&'a T);impl<'a,T>Deref for Guard<'a,T>{type Target=T;fn deref(&self)->&T{self.0}}
impl<'a,T>Guard<'a,T>{fn clone(other:&Self)->Self{Guard(other.0)}}fn check<T:Clone>(x:Guard<'_,T>)->T{x.clone()}
