fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn make()->(impl Value,impl Copy){(7u8,true)}fn main(){let x=make();check(x.0.value()==7);}
