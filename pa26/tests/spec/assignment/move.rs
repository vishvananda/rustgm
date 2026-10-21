fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);struct R(u32);impl core::ops::AddAssign<R> for C{fn add_assign(&mut self,r:R){self.0+=r.0;}}fn main(){let mut c=C(3);let r=R(4);c+=r;c+=r;}
