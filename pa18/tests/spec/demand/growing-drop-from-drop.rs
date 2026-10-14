#![recursion_limit="64"]struct D<T>([T;0]);impl<T> Drop for D<T>{fn drop(&mut self){let _d=D::<(T,)>([]);}}fn main(){let _d=D::<u8>([]);}
