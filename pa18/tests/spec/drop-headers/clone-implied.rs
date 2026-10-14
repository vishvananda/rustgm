struct S<T:Copy>(T);impl<T:Copy+Clone> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
