trait Tr<'r>{}
impl Tr<'static> for u8{}
struct S<F>(F);
impl<F:for<'r>Tr<'r>>S<F>{fn new()->u8{7}}
fn permitted()where u8:for<'r>Tr<'r>{let _=S::<u8>::new();}
fn main(){}
