struct S<'a,'b>(&'a u8,&'b u8);impl<'x,'y> Drop for S<'y,'x>{fn drop(&mut self){}}fn main(){}
