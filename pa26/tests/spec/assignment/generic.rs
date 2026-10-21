fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::AddAssign<u32> for C{fn add_assign(&mut self,r:u32){self.0+=r;}}fn apply<T:core::ops::AddAssign<u32>>(x:&mut T){*x+=4u32;}fn main(){let mut c=C(3);apply(&mut c);check(c.0==7);}
