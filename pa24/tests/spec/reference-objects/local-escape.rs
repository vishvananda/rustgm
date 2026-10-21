fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let p:&dyn Fn(u8)->u8;{let n=5u8;let f=move|x:u8|n+x;let r=&f;p=&r;}let _=p(2);}
