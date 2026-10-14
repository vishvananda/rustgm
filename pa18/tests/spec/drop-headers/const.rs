struct S<const N:usize>([u8;N]);impl<const N:usize> Drop for S<N>{fn drop(&mut self){}}
fn main(){}
