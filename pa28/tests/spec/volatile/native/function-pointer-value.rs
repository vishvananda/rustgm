fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u8)->u8{x+1}fn main(){let p:fn(u8)->u8=inc;unsafe{let f=core::ptr::read_volatile(&raw const p);check(f(7)==8);}}
