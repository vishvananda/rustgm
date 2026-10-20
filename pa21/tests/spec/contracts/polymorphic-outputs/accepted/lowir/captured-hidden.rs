fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read<'a>{fn read(&self,x:&'a u8)->u8;}struct S<'b>(&'b u8);impl<'a,'b>Read<'a> for S<'b>{fn read(&self,x:&'a u8)->u8{*self.0+*x}}fn f<'b>(x:&'b u8)->impl for<'a>Read<'a>+use<'b>{S(x)}fn main(){let a=7;let b=11;check(f(&a).read(&b)==18);}
