fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{type Out;}fn apply<'a,T:A<'a>,U:A<'a>>(f:fn(T::Out)->T::Out,x:U::Out)->T::Out{f(x)}fn main(){}