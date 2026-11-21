struct S<F>(F);
impl<F:FnMut(&u8)->&'static u8>S<F>{fn get(&mut self)->u8{7}}
trait Mention{fn get(&mut self)->u16;}
impl Mention for S<fn(&u8)->&u8>{fn get(&mut self)->u16{9}}
fn same(x: &u8)->&u8{x}
fn main(){let mut s=S(same as fn(&u8)->&u8);let _:u16=s.get();}
