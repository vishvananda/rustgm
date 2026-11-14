#![feature(const_trait_impl,const_convert)]
#![no_std]
use core::ops::{Deref,DerefMut};
pub struct Smart<T>(T);
impl<T> Deref for Smart<T>{type Target=T;fn deref(&self)->&T{&self.0}}
impl<T> DerefMut for Smart<T>{fn deref_mut(&mut self)->&mut T{&mut self.0}}
pub struct Pair<T>{inner:T}
pub const fn field<T>(x:&Smart<T>)->&T{Deref::deref(x)}
