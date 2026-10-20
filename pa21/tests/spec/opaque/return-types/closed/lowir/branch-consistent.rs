fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn make(b:bool)->impl Value{if b{7u8}else{11u8}}fn main(){check(make(false).value()==11);}
