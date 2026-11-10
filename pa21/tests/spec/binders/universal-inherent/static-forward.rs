trait Tr<'r>{}
impl Tr<'static> for u8{}
struct S<F>(F);
impl<F:for<'r>Tr<'r>>S<F>{fn new()->u8{7}}
fn bad()->u8{S::<u8>::new()}
fn main(){let _=bad();}
