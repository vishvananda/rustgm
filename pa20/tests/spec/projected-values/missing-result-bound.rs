fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word:Copy{const ZERO:Self;}trait Float{type Int:Copy;}fn value<T:Float>()->T::Int{T::Int::ZERO}fn main(){}
