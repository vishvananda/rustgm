fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait CastInto<T:Copy>:Copy{fn cast(self)->T;}impl CastInto<i16> for u8{fn cast(self)->i16{self as i16}}trait Int:Copy+CastInto<i16>{}impl Int for u8{}trait Float{type Int:Int;}struct F;impl Float for F{type Int=u8;}fn conv<T:Float>(x:T::Int)->u16{x.cast()}fn main(){check(conv::<F>(7)==7);}
