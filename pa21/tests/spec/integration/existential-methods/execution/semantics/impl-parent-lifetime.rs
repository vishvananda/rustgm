fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make(self)->impl Copy;}struct S<'a>(&'a u8);impl<'a>Make for S<'a>{fn make(self)->&'a u8{self.0}}fn main(){let x=71;check(*S(&x).make()==71);}
