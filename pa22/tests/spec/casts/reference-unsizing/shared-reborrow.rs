fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[3u8,5];let r=&mut a;let s=r as &[u8];check(s[0]==3);r[1]=9;check(a[1]==9);}
