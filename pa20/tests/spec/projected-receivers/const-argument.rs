fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word<const N:usize>{fn value(&self)->u8;}impl Word<7> for u8{fn value(&self)->u8{*self}}trait Float{type Int:Word<7>;}struct F;impl Float for F{type Int=u8;}fn conv<T:Float>(x:T::Int)->u8{x.value()}fn main(){check(conv::<F>(7)==7);}
