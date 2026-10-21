fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=7u8;let f=move|n:u8|x+n;let p:&dyn Fn(u8)->u8=&f;check(p(2)==9);check(p(3)==10);}
