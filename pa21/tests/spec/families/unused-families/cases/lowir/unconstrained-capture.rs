trait Q{type Out<'a>;}struct S;impl<'p>Q for S{type Out<'a>=&'p u8;}
fn main(){}
