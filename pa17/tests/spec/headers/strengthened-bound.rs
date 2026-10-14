struct S<'a,'b>(&'a u8,&'b u8);impl<'a:'b,'b> Drop for S<'a,'b>{fn drop(&mut self){}}fn main(){}
