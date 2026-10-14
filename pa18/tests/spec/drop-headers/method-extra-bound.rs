struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self)where T:Copy{}}
fn main(){}
