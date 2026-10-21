fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<'a>Fn(fn(&u8)->&u8,&'a u8)->&'a u8>(f:F)->F{f}fn id(x:&u8)->&u8{x}fn main(){let f=infer(|g,x|g(x));let n=7;check(*f(id,&n)==7);}
