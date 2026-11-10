fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word:Sized{const VALUE:u8;}struct Token(u8);impl Word for Token{const VALUE:u8=7;}trait Float{type Int:Word;}struct F;impl Float for F{type Int=Token;}fn value<T:Float>()->u8{T::Int::VALUE}fn main(){check(value::<F>()==7);}
