fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=7;let r=&n;let f=move||r;check(*f()==7u8);check(*f()==7u8);}
