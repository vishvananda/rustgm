struct S<'a:'b,'b>(&'a u8,&'b u8);impl<'a,'b> Drop for S<'a,'b>{fn drop(&mut self) where 'a:'b{}}fn main(){}
