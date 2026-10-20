fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make<'a>(x:&'a u8)->impl Copy+'a;}impl Make for u8{fn make<'a>(x:&'a u8)->&'a u8{x}}fn main(){let x=53;check(*<u8 as Make>::make(&x)==53);}
