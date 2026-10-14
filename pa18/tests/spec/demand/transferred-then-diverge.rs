#![recursion_limit="64"]struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){grow::<T>();}}fn grow<T>(){grow::<(T,)>();}fn id<T>(x:T)->T{x}fn main(){let _d=id(D(7u8));loop{}}
