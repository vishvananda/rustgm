struct S<'a>(&'a u8);impl<'a:'b,'b> Drop for S<'a>{fn drop(&mut self){}}fn main(){}
