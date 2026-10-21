fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::BitOrAssign<u8> for C{fn bitor_assign(&mut self,r:u8){self.0 |= r as u32;}}fn main(){let mut c=C(10);c |= 6u8;check(c.0==14);}
