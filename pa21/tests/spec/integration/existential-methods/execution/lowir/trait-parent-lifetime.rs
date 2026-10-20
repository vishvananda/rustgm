fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make<'a>{fn make(x:&'a u8)->impl Copy+'a;}impl<'a>Make<'a> for u16{fn make(x:&'a u8)->&'a u8{x}}fn main(){let x=73;check(*<u16 as Make>::make(&x)==73);}
