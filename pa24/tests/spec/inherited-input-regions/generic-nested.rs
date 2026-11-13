fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'b,'c,T>(x:&'c &'b T)->&'b T{(|v|(|&s|s)(v))(x)}
fn main(){let x=7u8;let p=&x;check(*f(&p)==7);}
