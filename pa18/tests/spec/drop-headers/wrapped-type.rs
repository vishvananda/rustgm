struct S<T>(T);impl<T> Drop for S<[T;1]>{fn drop(&mut self){}}
fn main(){}
