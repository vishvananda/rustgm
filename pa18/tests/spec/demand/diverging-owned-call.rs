#![recursion_limit="64"]struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){grow::<T>();}}fn grow<T>(){grow::<(T,)>();}fn keep<T>(x:T)->!{loop{}}fn main(){keep(D(7u8));}
