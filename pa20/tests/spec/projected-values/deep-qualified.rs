fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word:Copy{const ZERO:Self;}impl Word for u8{const ZERO:u8=3;}trait Middle{type Repr:Word;}impl Middle for u16{type Repr=u8;}trait Float{type Int:Middle;}struct F;impl Float for F{type Int=u16;}fn value<T:Float>()-><T::Int as Middle>::Repr{<T::Int as Middle>::Repr::ZERO}fn main(){check(value::<F>()==3);}
