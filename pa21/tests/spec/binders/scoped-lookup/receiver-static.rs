trait Tr<'r>{}
impl Tr<'static>for u8{}
struct S<F>(F);
impl<F:for<'r>Tr<'r>>S<F>{fn get(&self)->u8{7}}
trait Mention{fn get(&self)->u16;}
impl Mention for S<u8>{fn get(&self)->u16{9}}
fn main(){let _:u16=S(0u8).get();}
