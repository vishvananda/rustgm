fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Read<'a>{fn read(&self,x:&'a u8)->u8;}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}trait Make{fn make()->impl for<'a>Read<'a>;}impl Make for u16{fn make()->impl for<'a>Read<'a>{3u8}}fn run<T:Make>()->u8{let x=7;T::make().read(&x)}fn main(){check(run::<u16>()==10);}
