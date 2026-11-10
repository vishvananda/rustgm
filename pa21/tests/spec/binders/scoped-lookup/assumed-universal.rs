trait Tr<'r>{}
impl Tr<'static>for u8{}
struct S<F>(F);
impl<F:for<'r>Tr<'r>>S<F>{fn new()->u8{7}}
trait Mention{fn new()->u16;}
fn permitted()where for<'r>u8:Tr<'r>{let _:u8=S::<u8>::new();}
fn main(){}
