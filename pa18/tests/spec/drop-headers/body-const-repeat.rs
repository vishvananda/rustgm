struct S<const N:usize>([u8;N]);impl<const N:usize> Drop for S<N>{fn drop(&mut self){let a=[3u8;N];self.0=a;}}
fn main(){}
