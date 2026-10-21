fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
impl PartialEq<u16> for u8{fn eq(&self,r:&u16)->bool{*self as u16==*r}}fn main(){}
