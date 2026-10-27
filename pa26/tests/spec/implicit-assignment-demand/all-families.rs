fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let y=3u32;let mut x=12u32;x=12u32;x+=&y;check(x==15u32);x=12u32;x-=&y;check(x==9u32);x=12u32;x*=&y;check(x==36u32);x=12u32;x/=&y;check(x==4u32);x=12u32;x%=&y;check(x==0u32);x=12u32;x&=&y;check(x==0u32);x=12u32;x|=&y;check(x==15u32);x=12u32;x^=&y;check(x==15u32);x=12u32;x<<=&y;check(x==96u32);x=12u32;x>>=&y;check(x==1u32);}
