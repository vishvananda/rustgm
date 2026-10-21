fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C;impl<T> core::ops::AddAssign<C> for &mut T{fn add_assign(&mut self,_r:C){}}fn main(){}
