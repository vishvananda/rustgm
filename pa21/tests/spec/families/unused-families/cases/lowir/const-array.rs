trait Q{type Out<const N:usize>;}struct S;impl Q for S{type Out<const M:usize>=[u8;M];}
fn main(){}
