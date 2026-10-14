struct S<const N:usize,const M:usize>;impl<const N:usize> Drop for S<N,N>{fn drop(&mut self){}}
fn main(){}
