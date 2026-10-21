fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static N:u32=4;fn apply<T:core::ops::AddAssign<&'static u32>>(x:&mut T,r:&'static u32){*x+=r;}fn main(){let mut x=3u32;let r:&'static u32=&N;apply(&mut x,r);check(x==7);}
