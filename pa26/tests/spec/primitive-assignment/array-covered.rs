fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C;impl<T> core::ops::AddAssign<C> for [T;1]{fn add_assign(&mut self,_r:C){}}fn main(){let mut x=[3u32];x+=C;check(x[0]==3);}
