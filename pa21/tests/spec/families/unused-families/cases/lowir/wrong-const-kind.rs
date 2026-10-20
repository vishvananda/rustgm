trait Q{type Out<const N:usize>;}struct S;impl Q for S{type Out<const M:u8>=u8;}
fn main(){}
