fn probe()->u8{let x=if true{7u8}else{return 0};x}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){check(probe()==7);}
