fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<'a,T:core::ops::AddAssign<&'a u32>>(x:&mut T,r:&'a u32){*x+=r;}fn main(){let mut x=3u32;let y=4u32;apply(&mut x,&y);check(x==7);}
