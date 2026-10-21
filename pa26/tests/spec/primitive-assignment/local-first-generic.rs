fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl<T> core::ops::AddAssign<T> for C{fn add_assign(&mut self,_r:T){self.0+=1;}}fn main(){let mut c=C(3);c+=4u32;check(c.0==4);}
