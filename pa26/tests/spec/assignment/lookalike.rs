fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait AddAssign<R>{fn add_assign(&mut self,r:R);}struct C(u32);impl AddAssign<u32> for C{fn add_assign(&mut self,r:u32){self.0+=r;}}fn main(){let mut c=C(3);c+=4u32;}
