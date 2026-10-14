struct S<'a,T>(&'a T);impl<'a,'x,T> Drop for S<'a,T>where T:'x,'x:'a{fn drop(&mut self){}}
fn main(){}
