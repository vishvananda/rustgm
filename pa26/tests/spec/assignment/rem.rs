fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::RemAssign<u8> for C{fn rem_assign(&mut self,r:u8){self.0 %= r as u32;}}fn main(){let mut c=C(8);c %= 3u8;check(c.0==2);}
