fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<'a,F:for<'b>Fn(&'b u8)->&'a u8>(f:F)->F{f}fn main(){let n=7u8;let r=&n;let f=infer(move|_|r);let x=3;check(*f(&x)==7);}
