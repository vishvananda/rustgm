struct S<'a,'b>(&'a u8,&'b u8);impl<'a> Drop for S<'a,'_>{fn drop(&mut self){}}fn main(){}
