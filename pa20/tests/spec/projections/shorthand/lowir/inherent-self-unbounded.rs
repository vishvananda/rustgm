fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}struct S;impl A for S{type Out=u16;}impl S{fn f()->Self::Out{7}}fn main(){}