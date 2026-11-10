trait Tr<'r>{}
impl Tr<'static>for u8{}
struct S<F>(F);
impl<F:for<'r>Tr<'r>>S<F>{fn new()->u8{7}}
trait Mention{fn new()->u16;}
impl Mention for S<u8>{fn new()->u16{9}}
fn main(){let _:u16=S::<u8>::new();}
