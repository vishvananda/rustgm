struct S<'a,T,U>(&'a T,U);impl<'a,A,B> Drop for S<'a,B,A>where A:'a{fn drop(&mut self){}}
fn main(){}
