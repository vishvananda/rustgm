fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read<T>{fn value(&self)->u8;}impl<'a>Read<fn(&'a u8)> for u8{fn value(&self)->u8{*self}}fn f()->impl for<'a>Read<fn(&'a u8)>{31u8}fn take<T:for<'a>Read<fn(&'a u8)>>(x:T)->u8{x.value()}fn main(){check(take(f())==31);}
