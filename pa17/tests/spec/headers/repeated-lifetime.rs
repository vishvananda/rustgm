struct S<'a,'b>(&'a u8,&'b u8);impl<'a> Drop for S<'a,'a>{fn drop(&mut self){}}fn main(){}
