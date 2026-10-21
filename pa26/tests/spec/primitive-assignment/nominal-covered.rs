fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C;impl<T> core::ops::AddAssign<C> for core::option::Option<T>{fn add_assign(&mut self,_r:C){}}fn main(){let mut x:core::option::Option<u32>=None;x+=C;check(true);}
