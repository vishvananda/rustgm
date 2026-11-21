struct S<F>(F);
impl<F:FnMut(&u8)->&'static u8>S<F>{fn new()->u8{7}}
fn main(){let _:u8=S::<fn(&u8)->&'static u8>::new();}
