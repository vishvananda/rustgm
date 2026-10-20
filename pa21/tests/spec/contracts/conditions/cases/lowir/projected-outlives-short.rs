trait A<'a>{type Out;}impl<'a,'x>A<'a>for &'x u8{type Out=&'x u8;}fn f<T>()where for<'a>T:A<'a>,for<'a><T as A<'a>>::Out:'a{}fn run<'x>(x:&'x u8){f::<&'x u8>();}fn main(){let n=7;run(&n);}
