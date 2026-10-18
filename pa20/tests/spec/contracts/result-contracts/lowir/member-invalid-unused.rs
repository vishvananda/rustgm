fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a>{type Out:'a;}struct S<'b>(&'b u8);impl<'a,'b>A<'a>for S<'b>{type Out=&'b u8;}fn main(){}