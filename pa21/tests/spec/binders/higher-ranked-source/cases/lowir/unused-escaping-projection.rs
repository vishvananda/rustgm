trait A<'a> { type Out; }
fn f<'b,T>(x:<T as A<'b>>::Out)->&'static u8 where for<'a>T:A<'a>{let n=7u8;&n}
fn main(){}
