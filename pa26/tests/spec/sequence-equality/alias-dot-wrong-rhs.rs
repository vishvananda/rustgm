fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A=[u8;2];struct Wrong;fn main(){let a:A=[1,2];let _=a.eq(&Wrong);}
