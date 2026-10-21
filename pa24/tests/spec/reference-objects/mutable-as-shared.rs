fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=5u8;let mut f=|x:u8|{n+=x;n};let r=&mut f;let p:&dyn Fn(u8)->u8=&r;let _=p(2);}
