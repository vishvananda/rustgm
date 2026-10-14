struct S<'a,T>(&'a T);impl<T> Drop for S<'_,T>{fn drop(&mut self){let f:for<'x> fn(&'x Self);}}
fn main(){}
