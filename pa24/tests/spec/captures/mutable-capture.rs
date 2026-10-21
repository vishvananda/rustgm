fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=7u8;{let mut f=||{x+=2;x};check(f()==9);check(f()==11);}check(x==11);}
