struct S<T,U>(T,U);impl<T> Drop for S<T,T>{fn drop(&mut self){}}
fn main(){}
