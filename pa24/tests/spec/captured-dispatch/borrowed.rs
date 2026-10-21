fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn use_it(x:&u8)->u8{let f=||*x;let p:&dyn Fn()->u8=&f;p()}fn main(){check(use_it(&7)==7);}
