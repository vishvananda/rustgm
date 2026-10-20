fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a,'b>(x:&'a u8,y:&'b u8)->(impl Value,impl Value){(x,y)}fn main(){let x=29;let y=31;let pair=f(&x,&y);check(pair.0.value()==29);check(pair.1.value()==31);}
