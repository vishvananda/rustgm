trait Q{type Out<T>;}struct S;impl Q for S{type Out<const N:usize>=u8;}
fn main(){}
