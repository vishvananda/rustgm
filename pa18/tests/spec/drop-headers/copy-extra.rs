struct S<T>(T);impl<T:Copy> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
