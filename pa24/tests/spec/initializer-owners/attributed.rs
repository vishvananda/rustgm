fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[allow(unused_parens)]const F:fn(&u8)->&u8=((|x|x));fn main(){let x=9;check(*F(&x)==9);}
