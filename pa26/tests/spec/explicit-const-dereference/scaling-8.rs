#![feature(const_trait_impl,const_convert)]
#![no_std]
use core::ops::{Deref,DerefMut};
pub struct Smart<T>(T);
const impl<T> Deref for Smart<T>{type Target=T;fn deref(&self)->&T{&self.0}}
const impl<T> DerefMut for Smart<T>{fn deref_mut(&mut self)->&mut T{&mut self.0}}
pub struct Pair<T>{inner:T}
pub const fn field_0<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_1<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_2<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_3<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_4<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_5<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_6<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_7<T>(x:&Smart<T>)->&T{&**x}
fn main(){}
