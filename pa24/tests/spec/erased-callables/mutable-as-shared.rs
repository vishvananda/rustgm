fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=0u8;let f=||{n+=1;n};let g:&dyn Fn()->u8=&f;g();}
