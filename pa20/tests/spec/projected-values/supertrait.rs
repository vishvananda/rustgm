fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Base:Copy{const ZERO:Self;}trait Word:Base{}impl Base for u8{const ZERO:u8=3;}impl Word for u8{}trait Float{type Int:Word;}struct F;impl Float for F{type Int=u8;}fn value<T:Float>()->T::Int{T::Int::ZERO}fn main(){check(value::<F>()==3);}
