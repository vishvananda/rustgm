trait Q{type Out<T:?Sized+Copy>;}struct S;impl Q for S{type Out<T:Copy>=u8;}
fn main(){}
