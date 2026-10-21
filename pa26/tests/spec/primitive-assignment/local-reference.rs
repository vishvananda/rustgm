fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::AddAssign<u32> for &mut C{fn add_assign(&mut self,r:u32){self.0+=r;}}fn main(){let mut c=C(3);let mut r=&mut c;r+=4u32;check(c.0==7);}
