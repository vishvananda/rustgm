fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut a=[3u32,5];{let r=&mut a;let mut f=||{r[1]+=2;};f();f();}check(a[1]==9);}
