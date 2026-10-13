mod m{pub struct S<'a>(pub &'a u8);}fn f<'a>(x:&'a u8)->m::S<'a>{m::<'a>::S(x)}
fn main() {}
