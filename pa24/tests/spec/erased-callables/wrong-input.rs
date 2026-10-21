fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f=|x:u8|x;let g:&dyn Fn(u16)->u8=&f;g(7);}
