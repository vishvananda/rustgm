fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=3u32;<u32 as core::ops::AddAssign>::add_assign(&mut a,4);check(a==7);}
