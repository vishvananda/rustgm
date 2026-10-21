fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f=|x:u8|x+2;let p:&dyn Fn(u8)->u8=&f;check(p(7)==9);}
