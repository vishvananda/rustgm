fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:&dyn Fn(&u8)->&u8=&|x|x;let n=7;check(*f(&n)==7);}
