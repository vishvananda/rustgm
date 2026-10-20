fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Get<'a>{type Item;fn get(&self,x:&'a u8)->Self::Item;}impl<'a>Get<'a> for u8{type Item=&'a u8;fn get(&self,x:&'a u8)->&'a u8{x}}fn f()->impl for<'a>Get<'a,Item=&'a u8>{7u8}fn main(){let a=23;check(*f().get(&a)==23);}
