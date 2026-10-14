struct S<T:Copy>(T);impl<T> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
