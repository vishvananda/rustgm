fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S(u8);fn infer<F:for<'a>FnOnce(&'a u8)->&'a u8>(f:F)->F{f}fn main(){let s=S(2);let f=infer(move |x|{let _m=s;x});let x=7u8;check(*f(&x)==7);}
