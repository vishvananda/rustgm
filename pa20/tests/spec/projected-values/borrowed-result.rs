fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word<'a>{const VALUE:&'a u8;}impl<'a> Word<'a> for u8{const VALUE:&'a u8=&7;}trait Float<'a>{type Int:Word<'a>;}struct F;impl<'a> Float<'a> for F{type Int=u8;}fn value<'a,T:Float<'a>>()->&'a u8{T::Int::VALUE}fn main(){check(*value::<F>()==7);}
