struct S<'a,T>(&'a T);impl<T> Drop for S<'_,T>where Self:Sized{fn drop(&mut self){}}
fn main(){}
