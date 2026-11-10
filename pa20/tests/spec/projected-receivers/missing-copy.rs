fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Invoke{fn go(self)->u8 where Self:Copy;}impl Invoke for u8{fn go(self)->u8{self}}trait Float{type Int:Invoke;}struct F;impl Float for F{type Int=u8;}fn conv<T:Float>(x:T::Int)->u8{x.go()}fn main(){check(conv::<F>(7)==7);}
