trait Mark{}trait Q{type Out<T>;}struct S;impl Q for S{type Out<T:Mark>=T;}
fn main(){}
