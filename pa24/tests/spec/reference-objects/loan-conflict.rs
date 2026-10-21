fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=5u8;let mut f=|x:u8|{n+=x;n};let mut r=&mut f;let p:&mut dyn FnMut(u8)->u8=&mut r;let _=f(1);let _=p(2);}
