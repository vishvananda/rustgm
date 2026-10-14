struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}fn main(){let mut n=7u8;let d=D(&n);n=8;}
