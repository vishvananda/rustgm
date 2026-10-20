fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make<T:Copy>(x:T)->impl Copy;}impl Make for u8{fn make<T:Copy>(x:T)->T{x}}fn main(){let x:u16=<u8 as Make>::make(47u16);check(x==47);}
