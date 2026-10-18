fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{type Out;}fn f<'a,'b,T:A<'a>+A<'b>>(x:T::Out)->T::Out{x}fn main(){}