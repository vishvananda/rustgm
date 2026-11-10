trait Tr<'r>{}
impl<'r>Tr<'r>for u8{}
struct S<F>(F);
impl<F:for<'r>Tr<'r>>S<F>{fn new()->u8{7}}
trait Mention{fn new()->u16;}
fn main(){let _:u8=S::<u8>::new();}
