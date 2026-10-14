struct S<T>(T);impl<T:?Sized> Drop for S<T>where T:Sized{fn drop(&mut self){}}
fn main(){}
