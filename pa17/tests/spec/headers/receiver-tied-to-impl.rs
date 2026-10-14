struct S<'a>(&'a u8);impl<'a> Drop for S<'a>{fn drop(&'a mut self){}}fn main(){}
