struct S<const N:usize,const M:usize>;impl<const M:usize,const N:usize> Drop for S<N,M>{fn drop(&mut self){}}
fn main(){}
