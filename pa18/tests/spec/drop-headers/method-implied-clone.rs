struct S<T:Copy>(T);impl<T:Copy> Drop for S<T>{fn drop(&mut self)where T:Clone{}}
fn main(){}
