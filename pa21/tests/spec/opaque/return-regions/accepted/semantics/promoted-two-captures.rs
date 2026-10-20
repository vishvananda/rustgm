fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a,'b>(_:&'a u8,_:&'b u8)->impl Value{&19u8}fn main(){let x=1;let y=2;check(f(&x,&y).value()==19);}
