fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
use core::ops::AddAssign;
fn main(){let mut x=3u32;let y=4u32;x+=&y;check(x==7);}
