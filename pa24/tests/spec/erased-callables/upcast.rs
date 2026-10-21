fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=4;let f=|x:u8|x+n;let g:&dyn Fn(u8)->u8=&f;let _h:&dyn FnMut(u8)->u8=g;}
