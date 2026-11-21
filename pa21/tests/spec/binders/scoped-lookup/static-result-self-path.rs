struct S<F>(F);
impl<F:FnMut(&u8)->&'static u8>S<F>{fn last()->u8{7}fn get(&mut self)->u8{Self::last()}}
fn seven(_: &u8)->&'static u8{&7}
fn main(){let mut s=S(seven as fn(&u8)->&'static u8);let _:u8=s.get();}
