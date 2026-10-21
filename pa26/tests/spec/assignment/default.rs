fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::AddAssign for C{fn add_assign(&mut self,r:C){self.0+=r.0;}}fn main(){let mut c=C(3);c+=C(4);check(c.0==7);}
