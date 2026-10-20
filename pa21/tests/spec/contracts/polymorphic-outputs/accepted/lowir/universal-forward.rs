fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read<'a>{fn read(&self,x:&'a u8)->u8;}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}fn f()->impl for<'a>Read<'a>{7u8}fn g()->impl for<'b>Read<'b>{f()}fn take<T:for<'c>Read<'c>>(x:T)->u8{let a=5;x.read(&a)}fn main(){check(take(g())==12);}
