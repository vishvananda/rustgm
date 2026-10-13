fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&((),u8))->u8{match x{(z,n) if {let _=&*z;*n==7}=>*n,_=>0}}fn main(){check(f(&((),7))==7);}
