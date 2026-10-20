fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Base{fn value(&self)->u8;}trait Derived:Base{}impl Base for u8{fn value(&self)->u8{*self}}impl Derived for u8{}fn f()->impl Derived{11u8}fn take<T:Base>(x:T)->u8{x.value()}fn main(){check(take(f())==11);}
