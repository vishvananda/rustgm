fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait PartialEq<Rhs=Self>{fn eq(&self,_:&Rhs)->bool;}struct X;impl PartialEq for X{fn eq(&self,_:&Self)->bool{true}}fn main(){let _=[X]==[X];}
