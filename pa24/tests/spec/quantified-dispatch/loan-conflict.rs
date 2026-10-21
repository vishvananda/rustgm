fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=1u8;let f=|x:&u8|n+*x;let p:&dyn Fn(&u8)->u8=&f;n=4;let _=p(&7);}
