fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u64)->u64{x+1}static F:fn(u64)->u64=inc;const G:fn(u64)->u64=inc;fn main(){check(F(7)==8 && G(9)==10);}
