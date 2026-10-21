fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Local{}impl core::ops::AddAssign<u32> for dyn Local{fn add_assign(&mut self,_r:u32){}}fn main(){}
