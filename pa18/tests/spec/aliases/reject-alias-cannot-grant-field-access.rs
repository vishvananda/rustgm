mod m{pub struct S{n:usize}pub type A<const N:usize>=[u8;N];}fn f(){let _:m::A<{m::S{n:3}.n}>;}
fn main(){}
