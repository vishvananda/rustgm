struct S<'a,T>(&'a T);impl<'a,T:'a> Drop for S<'a,T>{fn drop(&mut self){}}
fn main(){}
