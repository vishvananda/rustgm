struct S<T>(T);impl<T> Drop for S<T>{fn drop<U>(&mut self){}}
fn main(){}
