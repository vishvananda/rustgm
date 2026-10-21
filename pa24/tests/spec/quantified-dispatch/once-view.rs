fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let n=1u8;let f=move|x:&u8|n+*x;let p:&dyn FnOnce(&u8)->u8=&f;let _=p;check(f(&7)==8);}
