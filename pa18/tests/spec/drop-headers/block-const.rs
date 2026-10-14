struct S<const N:usize>;impl<const N:usize> Drop for S<{N}>{fn drop(&mut self){}}
fn main(){}
