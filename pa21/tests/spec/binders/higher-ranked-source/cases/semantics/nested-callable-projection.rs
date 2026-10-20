trait A<'a> { type Out; }
impl<'a>A<'a>for u8{type Out=&'a u8;}
fn f<T>(x:for<'a>fn(&'a u8)-><T as A<'a>>::Out)->for<'a>fn(&'a u8)-><T as A<'a>>::Out where for<'a>T:A<'a>{x}
fn id(x:&u8)->&u8{x}
fn main(){let n=7u8;let g=f::<u8>(id);if *g(&n)!=7{loop{}}}
