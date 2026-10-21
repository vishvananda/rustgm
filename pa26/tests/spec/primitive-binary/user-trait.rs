fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Sub{fn sub(self,r:u8)->u16;}impl Sub for u8{fn sub(self,r:u8)->u16{self as u16*10+r as u16}}fn marker<T:core::ops::Sub>(){}fn main(){check(7u8.sub(2)==72);check(<u8 as Sub>::sub(7,2)==72);check(<u8 as core::ops::Sub>::sub(7,2)==5);}
