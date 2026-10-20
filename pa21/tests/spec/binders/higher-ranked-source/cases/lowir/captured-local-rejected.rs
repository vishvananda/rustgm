trait A<'a> { type Out; }
struct S<'x>(&'x u8);
impl<'a,'x:'a> A<'a> for S<'x>{type Out=&'x u8;}
fn f<'b,T>(x:<T as A<'b>>::Out)-><T as A<'b>>::Out where for<'a>T:A<'a>{x}
fn run<'x>(x:&'x u8){let _=f::<S<'x>>(x);}fn main(){let x=7u8;run(&x);}
