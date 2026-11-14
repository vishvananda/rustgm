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
pub const fn field_8<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_9<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_10<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_11<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_12<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_13<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_14<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_15<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_16<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_17<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_18<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_19<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_20<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_21<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_22<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_23<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_24<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_25<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_26<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_27<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_28<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_29<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_30<T>(x:&Smart<T>)->&T{&**x}
pub const fn field_31<T>(x:&Smart<T>)->&T{&**x}
fn main(){}
