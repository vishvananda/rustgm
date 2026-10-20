fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a,'b:'a>(_:&'a u8,x:&'b u8)->impl Value+use<'a>{x}fn main(){let x=37;let y=1;check(f(&y,&x).value()==37);}
