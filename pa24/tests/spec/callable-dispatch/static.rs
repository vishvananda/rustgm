fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u8)->u8{x+2}const F:fn(u8)->u8=inc;const P:&dyn Fn(u8)->u8=&F;fn main(){check(P(7)==9);}
