trait A<const N:usize>{}impl<const N:usize>A<N> for [u8;N]{}trait Q{type Out<const N:usize>:A<N>;}struct S;impl Q for S{type Out<const M:usize>=[u8;M];}
fn main(){}
