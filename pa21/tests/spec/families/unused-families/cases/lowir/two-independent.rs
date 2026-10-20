trait Q{type Out<T:Copy,U>;}struct S;impl Q for S{type Out<T,U:Copy>=u8;}
fn main(){}
