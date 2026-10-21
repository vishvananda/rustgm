fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u8)->u8{x+2}fn main(){let mut f:fn(u8)->u8=inc;let p:&mut dyn Fn(u8)->u8=&mut f;let q:&mut dyn FnMut(u8)->u8=p;check(q(7)==9);}
