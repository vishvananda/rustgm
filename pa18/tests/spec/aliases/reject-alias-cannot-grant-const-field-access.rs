mod m{pub struct S{n:usize}pub const C:S=S{n:3};pub type A<const N:usize>=[u8;N];}fn f(){let _:m::A<{m::C.n}>;}
fn main(){}
