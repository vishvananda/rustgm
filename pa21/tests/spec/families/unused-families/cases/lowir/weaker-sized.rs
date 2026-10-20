trait Q{type Out<T>;}struct S;impl Q for S{type Out<T:?Sized>=u8;}
fn main(){}
