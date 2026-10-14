struct S<'a>(&'a u8);impl Drop for S{fn drop(&mut self){}}fn main(){}
