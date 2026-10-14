struct S<'a,T,const N:usize>(&'a T,[u8;N]);impl<'a,T,const N:usize> Drop for S<'a,T,N>{fn drop(&mut self){}}
fn main(){}
