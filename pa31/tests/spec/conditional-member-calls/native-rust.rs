#![feature(const_trait_impl,const_convert)]
#![no_std]
use core::ops::{Deref,DerefMut};
pub struct Smart<T>(T);
const impl<T> Deref for Smart<T>{type Target=T;fn deref(&self)->&T{&self.0}}
const impl<T> DerefMut for Smart<T>{fn deref_mut(&mut self)->&mut T{&mut self.0}}
pub struct Pair<T>{inner:T}
pub const fn shared<T>(x:&Smart<Pair<T>>)->&T{&x.inner}
pub const fn unique<T>(x:&mut Smart<Pair<T>>)->&mut T{&mut x.inner}
pub const fn direct<T>(x:&Smart<T>)->&T{Deref::deref(x)}
#[unsafe(no_mangle)]pub extern "C" fn entry()->u8{let mut x=Smart(Pair{inner:23u8});*unique(&mut x)=48;let y=Smart(shared(&x));**direct(&y)}
