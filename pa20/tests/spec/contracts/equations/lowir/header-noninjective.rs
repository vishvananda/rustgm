fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A where Self::Out:A<Out=Self::Out>{type Out;}fn wrong<T:A>(x:T)->T::Out{x}fn main(){}