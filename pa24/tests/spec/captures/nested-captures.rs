fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=7u8;let outer=||{let inner=||n;inner()+2};check(outer()==9);}
