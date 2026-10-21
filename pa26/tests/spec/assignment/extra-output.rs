fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C(u32);impl core::ops::AddAssign<u32> for C{type Output=u32;fn add_assign(&mut self,r:u32){}}fn main(){}
