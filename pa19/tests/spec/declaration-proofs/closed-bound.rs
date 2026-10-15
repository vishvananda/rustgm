fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{}impl A for u8{}fn value()->u8 where u8:A{7}fn main(){check(value()==7);}