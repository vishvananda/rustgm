fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a>(x:&'a u8)->impl Value{x}fn main(){let x=7;check(f(&x).value()==7);}
