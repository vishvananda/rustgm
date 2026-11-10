fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Word:Copy{const ZERO:Self;}impl Word for u8{const ZERO:u8=0;}trait Float{type Int:Word;fn zero()->Self::Int{<Self::Int as Word>::ZERO}}struct F;impl Float for F{type Int=u8;}fn main(){check(F::zero()==0);}
