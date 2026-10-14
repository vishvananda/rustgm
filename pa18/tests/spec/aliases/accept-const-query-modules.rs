type A<const N:usize>=[u8;N];mod m{pub type B=super::A<{1+2}>;}fn f(_:m::B,_:A<{5+1}>){}
fn main(){}
