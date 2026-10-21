fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u8)->u8{x+2}fn apply<F:FnOnce(u8)->u8>(f:F)->u8{f(7)}fn main(){let p:&dyn Fn(u8)->u8=&inc;check(apply(p)==9);check(apply(&p)==9);check(apply(&&p)==9);}
