fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u8)->u8{x+2}fn main(){let mut f:fn(u8)->u8=inc;let p:&mut dyn FnMut(u8)->u8=&mut f;check(p(7)==9);check(p(8)==10);}
