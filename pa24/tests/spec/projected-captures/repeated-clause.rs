fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick<'a>{type Out:Copy;}impl<'a> Pick<'a> for u8{type Out=&'a u8;}fn take<'a,T:Pick<'a>+Pick<'a>>(x:T::Out)->T::Out{let f=move||x;f()}fn main(){let n=7;check(*take::<u8>(&n)==7);}
