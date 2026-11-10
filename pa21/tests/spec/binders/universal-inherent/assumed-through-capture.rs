trait Aux<'a>{}
trait Tr<'r>{}
impl<'r,'a>Tr<'r> for u8 where u16:Aux<'a>{}
struct S<F>(F);
impl<F:for<'r>Tr<'r>>S<F>{fn new()->u8{7}}
fn permitted<'x>()where u16:Aux<'x>{let _=S::<u8>::new();}
fn main(){}
