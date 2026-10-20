fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait Make{fn make(&self)->impl Value{7u8}}struct S;impl Make for S{}fn main(){check(S.make().value()==7);}
