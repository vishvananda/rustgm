fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Shift<R>:Copy{type Output;fn shift(self,r:R)->Self::Output;}impl Shift<u32> for u8{type Output=u8;fn shift(self,r:u32)->u8{self<<r}}trait Float{type Int:Copy+Shift<u32,Output=Self::Int>;}struct F;impl Float for F{type Int=u8;}fn choose<T:Float>(a:T::Int)->T::Int{a.shift(1)}fn main(){check(choose::<F>(7)==14);}
