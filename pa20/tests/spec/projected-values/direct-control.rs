fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word:Copy{const ZERO:Self;}impl Word for u8{const ZERO:u8=0;}trait Float{type Int:Word;}struct F;impl Float for F{type Int=u8;}fn zero<T:Word>()->T{T::ZERO}fn main(){check(zero::<u8>()==0);}
