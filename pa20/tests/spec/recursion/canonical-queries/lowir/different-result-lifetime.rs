fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{type Out:A<'a>;}fn f<'a,'b,T:A<'a>>(x:<<T as A<'a>>::Out as A<'b>>::Out)-><<T as A<'a>>::Out as A<'b>>::Out{x}fn main(){}