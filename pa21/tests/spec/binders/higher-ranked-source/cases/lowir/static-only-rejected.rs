trait A<'a> { type Out; }
impl A<'static> for u8 { type Out=u8; }
fn f<T>(x:<T as A<'static>>::Out)-><T as A<'static>>::Out where for<'a>T:A<'a> { x }
fn main(){let _=f::<u8>(7);}
