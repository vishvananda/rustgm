struct S<T>(T);impl<T> Drop for S<T>{fn drop<'a>(&'a mut self)where &'a T:Copy{}}
fn main(){}
