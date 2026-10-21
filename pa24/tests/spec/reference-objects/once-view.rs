fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=5u8;let f=move|x:u8|n+x;let r=&f;let p:&dyn FnOnce(u8)->u8=&r;let _=p;check(f(2)==7);}
