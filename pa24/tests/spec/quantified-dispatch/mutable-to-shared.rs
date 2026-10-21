fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut n=1u8;let f=move|x:&u8|{n+=*x;n};let p:&dyn Fn(&u8)->u8=&f;let _=p;}
