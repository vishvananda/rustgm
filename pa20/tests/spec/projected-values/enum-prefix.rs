fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Factory{type Item;fn make()->Self::Item;}enum E{V(u8)}struct F;impl Factory for F{type Item=E;fn make()->Self::Item{Self::Item::V(7)}}fn main(){let x=match F::make(){E::V(x)=>x};check(x==7);}
