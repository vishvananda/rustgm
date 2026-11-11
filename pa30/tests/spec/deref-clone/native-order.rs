#![allow(dead_code)]
use core::ops::Deref;
unsafe extern "C"{fn mark(step:u8);}
struct Value(u8);impl Clone for Value{fn clone(&self)->Self{unsafe{mark(3)};Value(self.0)}}
struct Guard<'a,T>(&'a T);impl<'a,T>Deref for Guard<'a,T>{type Target=T;fn deref(&self)->&T{unsafe{mark(2)};self.0}}
impl<'a,T>Guard<'a,T>{fn clone(other:&Self)->Self{Guard(other.0)}}
struct Cell<T>(T);impl<T>Cell<T>{fn new(t:T)->Self{Cell(t)}fn borrow(&self)->Guard<'_,T>{unsafe{mark(1)};Guard(&self.0)}}
impl<T:Clone>Clone for Cell<T>{fn clone(&self)->Self{Cell::new(self.borrow().clone())}}
#[unsafe(no_mangle)]pub extern "C" fn code()->u8{let cell=Cell(Value(23u8));let cloned=cell.clone();cloned.0.0}
