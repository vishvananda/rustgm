fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[3u8,5];let r=&mut a;let s=r as &mut [u8];r[0]=9;check(s[0]==3);}
