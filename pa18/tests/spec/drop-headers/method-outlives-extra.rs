struct S<T>(T);impl<T> Drop for S<T>{fn drop(&mut self)where T:'static{}}
fn main(){}
