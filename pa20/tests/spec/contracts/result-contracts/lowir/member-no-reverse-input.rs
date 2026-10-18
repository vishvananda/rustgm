fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out:'static;}fn require<T:'static>(x:T)->T{x}fn use_it<T:A>(x:T)->T{require(x)}fn main(){}