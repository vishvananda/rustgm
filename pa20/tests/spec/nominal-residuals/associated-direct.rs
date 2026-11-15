#![allow(dead_code,unused_variables)]
pub struct Key;
pub trait Pick<C:?Sized>{type Output:?Sized;fn get(self,input:Option<u8>,slice:&C)->Option<&Self::Output>;}
impl<T>Pick<[T]>for Key{type Output=[T];fn get(self,input:Option<u8>,slice:&[T])->Option<&Self::Output>{input?;Some(slice)}}
fn main(){}
