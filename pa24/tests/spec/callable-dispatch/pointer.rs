fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u8)->u8{x+2}fn main(){let f:fn(u8)->u8=inc;let p:&dyn Fn(u8)->u8=&f;check(p(7)==9);}
