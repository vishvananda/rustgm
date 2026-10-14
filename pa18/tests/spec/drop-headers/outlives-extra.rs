struct S<T>(T);impl<T:'static> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
