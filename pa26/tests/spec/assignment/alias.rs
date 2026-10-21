fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::AddAssign as Acc;struct C(u32);impl Acc<u32> for C{fn add_assign(&mut self,r:u32){self.0+=r;}}fn main(){let mut c=C(3);c+=4u32;check(c.0==7);}
