fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::AddAssign<u32> for C{fn add_assign(&mut self,r:u32){self.0+=r;}}struct W{c:C}fn main(){let mut w=W{c:C(3)};w.c+=4u32;check(w.c.0==7);}
