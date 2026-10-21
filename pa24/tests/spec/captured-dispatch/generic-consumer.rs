fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn call<F:FnOnce(u8)->u8>(f:F)->u8{f(2)}fn main(){let x=7u8;let f=move|n:u8|x+n;let p:&dyn Fn(u8)->u8=&f;check(call(p)==9);check(call(&p)==9);}
