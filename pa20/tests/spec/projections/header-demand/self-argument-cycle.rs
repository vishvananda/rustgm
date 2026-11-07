trait Tr<A> { type Out; }
impl Tr<Self::Out> for u8 { type Out = u8; }
fn main() {}
