fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<F:for<'a>Fn(&'a u8)->u8>(f:F)->u8{f(&7)}fn main(){let n=1u8;let f=move|x:&u8|n+*x;let p:&dyn Fn(&u8)->u8=&f;check(apply(p)==8);}
