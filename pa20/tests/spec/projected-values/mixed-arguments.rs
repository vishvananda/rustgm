fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word<T,const N:usize>{const VALUE:T;}impl Word<u8,3> for u16{const VALUE:u8=7;}trait Float{type Int:Word<u8,3>;}struct F;impl Float for F{type Int=u16;}fn value<T:Float>()->u8{T::Int::VALUE}fn main(){check(value::<F>()==7);}
