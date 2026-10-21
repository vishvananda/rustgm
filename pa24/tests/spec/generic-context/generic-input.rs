fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<T,F:for<'a>Fn(&'a T)->&'a T>(f:F,x:T)->F{let _=x;f}fn main(){let f=infer(|x|x,7u8);let n=9u8;check(*f(&n)==9);}
