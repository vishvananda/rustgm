fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C;trait Pick{type Out;}impl Pick for (){type Out=C;}impl core::ops::AddAssign<u32> for <() as Pick>::Out{fn add_assign(&mut self,_r:u32){}}fn main(){}
