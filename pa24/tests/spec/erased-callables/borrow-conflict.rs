fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=0u8;let mut f=||{n+=1;n};let g:&mut dyn FnMut()->u8=&mut f;n=7;g();}
