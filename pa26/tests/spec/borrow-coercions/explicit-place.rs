#![allow(dead_code,unused_variables)]
use core::ops::{Deref,DerefMut};
struct Hold<T>(T);impl<T>Deref for Hold<T>{type Target=T;fn deref(&self)->&T{&self.0}}
impl<T>DerefMut for Hold<T>{fn deref_mut(&mut self)->&mut T{&mut self.0}}
fn accept<T>(_:&mut T,t:T)->T{t}
fn check<T>(mut x:Hold<T>,t:T)->T{accept(&mut *x,t)}
fn main(){}
