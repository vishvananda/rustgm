struct S<T>(T);impl<'a,T> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
