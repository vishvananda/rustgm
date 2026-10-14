struct S<'a,'b>(&'a &'b u8);impl<'a,'b:'a> Drop for S<'a,'b>{fn drop(&mut self){}}fn main(){}
