struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self){}}impl<U> Drop for S<U>{fn drop(&mut self){}}
fn main(){}
