fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Sub as Canonical;trait Sub{fn sub(self,r:u8)->u16;}impl Sub for u8{fn sub(self,r:u8)->u16{self as u16*10+r as u16}}fn main(){let _=7u8.sub(2);}
