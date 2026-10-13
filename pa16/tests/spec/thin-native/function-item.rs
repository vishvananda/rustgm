fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f()->u8{7}fn main(){let r=&f;check((*r)()==7);}
