struct S<'a,'b>(&'a u8,&'b u8);impl Drop for S<'_,'_>{fn drop(&mut self){}}fn main(){}
