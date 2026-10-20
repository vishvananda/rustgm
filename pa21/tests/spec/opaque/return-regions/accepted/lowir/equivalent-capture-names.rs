fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a:'b,'b:'a>(x:&'a u8)->(impl Value+use<'a>,impl Value+use<'b>){(x,x)}fn main(){let x=41;let pair=f(&x);check(pair.0.value()==41);check(pair.1.value()==41);}
