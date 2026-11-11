#![allow(dead_code)]
use core::ops::Deref;
struct Guard<'a,T>(&'a T);impl<'a,T>Deref for Guard<'a,T>{type Target=T;fn deref(&self)->&T{self.0}}
struct Cell<T>(T);impl<T>Cell<T>{fn new(t:T)->Self{Cell(t)}fn borrow(&self)->Guard<'_,T>{Guard(&self.0)}}
impl<T:Clone>Clone for Cell<T>{fn clone(&self)->Self{Cell::new(self.borrow().clone())}}
