fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<'a>Fn(&'a u8)->&'a u8>(x:(F,))->F{x.0}fn main(){let f=infer((|x|x,));let n=7;check(*f(&n)==7);}
