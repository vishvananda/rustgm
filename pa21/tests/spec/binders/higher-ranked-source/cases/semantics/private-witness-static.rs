trait A<'a> { type Out; }
struct S<'x>(&'x u8);
impl<'a,'w,'x:'w> A<'a> for S<'x> where 'w:'a {type Out=&'x u8;}
fn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}
static V:u8=7;fn main(){if *f::<S<'static>>(&V)!=7{loop{}}}
