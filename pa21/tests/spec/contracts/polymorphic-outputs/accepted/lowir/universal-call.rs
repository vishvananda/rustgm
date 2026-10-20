fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read<'a>{fn read(&self,x:&'a u8)->u8;}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}fn f()->impl for<'a>Read<'a>{7u8}fn main(){let a=3;check(f().read(&a)==10);}
