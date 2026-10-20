trait Q{type Out<T:?Sized>;}struct S;impl Q for S{type Out<T:?Sized>=u8;}
fn main(){}
