fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=7u8;{let mut f=|n:u8|{x+=n;x};let p:&mut dyn FnMut(u8)->u8=&mut f;check(p(2)==9);check(p(3)==12);}check(x==12);}
