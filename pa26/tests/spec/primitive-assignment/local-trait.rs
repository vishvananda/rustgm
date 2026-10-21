fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait AddAssign<R>{fn add_assign(&mut self,r:R);}impl AddAssign<u32> for u32{fn add_assign(&mut self,r:u32){*self+=r;}}fn main(){}
