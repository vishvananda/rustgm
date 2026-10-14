struct S<T>(T);impl<T> Drop for S<T>where (for<'a> fn(&'a T),):Copy{fn drop(&mut self){}}
fn main(){}
