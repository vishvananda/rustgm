fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<'a,'b>Fn(&'a u8,&'b u8)->&'a u8>(f:F)->F{f}fn main(){let f=infer(|x,y|{let _=y;x});let x=7;let y=9;check(*f(&x,&y)==7);}
