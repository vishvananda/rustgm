#![allow(dead_code,unused_variables)]
pub fn make<T>(slice:&[T])->Option<&[T]>{Some(slice)}
pub fn probe<T>(slice:&[T])->Option<&[T]>{make(slice)}
fn main(){}

pub struct Key;
pub trait Pick<C:?Sized>{type Output:?Sized;fn get(self,slice:&C)->Option<&Self::Output>;}
impl<T>Pick<[T]>for Key{type Output=[T];fn get(self,slice:&[T])->Option<&Self::Output>{make(slice)}}
