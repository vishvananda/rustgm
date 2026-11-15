#![allow(dead_code,unused_variables)]
pub fn make<T>(slice:&[T])->Option<&[T]>{Some(slice)}
pub struct Key;
pub trait Pick<C:?Sized>{type Output:?Sized;fn get(self,input:Option<u8>,slice:&C)->Option<&Self::Output>;}
impl<T>Pick<[T]>for Key{type Output=[T];fn get(self,input:Option<u8>,slice:&[T])->Option<&Self::Output>{input?;make(slice)}}
fn main(){}
