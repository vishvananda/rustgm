fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<>Fn(u8)->u8>(f:F)->F{f}fn main(){let f=infer(|x|x);check(f(7)==7);}
