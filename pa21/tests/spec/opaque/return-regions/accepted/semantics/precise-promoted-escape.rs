fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a>(_:&'a u8)->impl Value+use<>{&23u8}fn main(){let value;{let local=1;value=f(&local);}check(value.value()==23);}
