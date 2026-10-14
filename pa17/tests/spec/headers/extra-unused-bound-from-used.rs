struct S<'a>(&'a u8);impl<'a,'b:'a> Drop for S<'a>{fn drop(&mut self){}}fn main(){}
