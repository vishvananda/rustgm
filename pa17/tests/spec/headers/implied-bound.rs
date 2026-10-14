struct S<'a:'b,'b>(&'a u8,&'b u8);impl<'x,'y> Drop for S<'x,'y>{fn drop(&mut self){}}fn main(){}
