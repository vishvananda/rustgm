fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Change{fn change(&mut self)->u8;}impl Change for u8{fn change(&mut self)->u8{*self+=1;*self}}trait Float{type Int:Change;}struct F;impl Float for F{type Int=u8;}fn conv<T:Float>(x:&mut T::Int)->u8{x.change()}fn main(){let mut x=6;check(conv::<F>(&mut x)==7);check(x==7);}
