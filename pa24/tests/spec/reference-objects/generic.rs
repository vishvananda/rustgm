fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<F:Fn(u8)->u8>(x:&F)->u8{let p:&dyn Fn(u8)->u8=&x;p(2)}fn main(){let n=5u8;let f=move|x:u8|n+x;check(apply(&f)==7);}
