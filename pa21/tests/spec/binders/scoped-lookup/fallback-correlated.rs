trait Tr<'a,'b>{}
impl<'a>Tr<'a,'a>for u8{}
struct S<F>(F);
impl<F:for<'a,'b>Tr<'a,'b>>S<F>{fn new()->u8{7}}
trait Mention{fn new()->u16;}
impl Mention for S<u8>{fn new()->u16{9}}
fn main(){let _:u16=S::<u8>::new();}
