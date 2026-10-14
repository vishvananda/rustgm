struct S<'a>(&'a u8);impl<'a:'x,'x:'static> Drop for S<'a>{fn drop(&mut self){}}fn main(){}
