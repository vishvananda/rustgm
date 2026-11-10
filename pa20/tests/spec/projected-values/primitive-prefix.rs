fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Factory{type Item;fn value()->u8;}struct F;impl Factory for F{type Item=u8;fn value()->u8{Self::Item::MAX}}fn main(){check(F::value()==255);}
