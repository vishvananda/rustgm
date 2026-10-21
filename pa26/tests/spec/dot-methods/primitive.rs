fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
use core::ops::AddAssign;fn main(){let mut x=7u8;x.add_assign(2u8);verify(x==9);}
