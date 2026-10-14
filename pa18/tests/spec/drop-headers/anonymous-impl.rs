struct S<'a,T>(&'a T);impl<T> Drop for S<'_,T>{fn drop(self:&mut Self){}}
fn main(){}
