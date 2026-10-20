trait A<'a> { type Out; }
impl<'a> A<'a> for u8 { type Out=u8; }
fn f<T>(x:<T as A<'static>>::Out)-><T as A<'static>>::Out where for<'a>T:A<'a> { x }
fn main(){if f::<u8>(7)!=7{loop{}}}
