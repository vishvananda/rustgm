fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<'a,'unused>Fn(&'a u8)->&'a u8>(f:F)->F{f}fn main(){let f=infer(|x|x);let n=7;check(*f(&n)==7);}
