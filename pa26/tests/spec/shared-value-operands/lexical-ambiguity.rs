fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::Add;trait Other<R>{fn add(self,r:R)->u8;}impl Other<&u8> for u8{fn add(self,_:&u8)->u8{19}}fn main(){let b=5u8;let _=2u8.add(&b);}
