fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C;type L=C;impl core::ops::AddAssign<L> for u32{fn add_assign(&mut self,_r:L){*self+=1;}}fn main(){let mut x=3u32;x+=C;check(x==4);}
