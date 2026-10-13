fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=7u8;let r=&x;check(*r==7);x=9;check(x==9);}
