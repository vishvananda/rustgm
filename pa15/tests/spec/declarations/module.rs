mod m{pub struct S<'a>(pub &'a u8);}fn f<'a>(x:m::S<'a>)->&'a u8{x.0}
fn main() {}
