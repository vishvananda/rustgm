fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read<'a>{fn read(&self,x:&'a u8)->u8;}trait All:for<'a>Read<'a>{}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}impl All for u8{}fn f()->impl All{7u8}fn take<T:for<'b>Read<'b>>(x:T)->u8{let a=5;x.read(&a)}fn main(){check(take(f())==12);}
