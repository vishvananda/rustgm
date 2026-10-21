fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct R(u32);impl core::ops::AddAssign<R> for u32{fn add_assign(&mut self,r:R){*self+=r.0;}}fn main(){let mut c=3u32;c+=R(4);check(c==7);}
