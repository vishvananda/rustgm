fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<'a>(x:&'a u8)->&'a u8{x}fn main(){let g:for<'a> fn(&'a u8)->&'a u8=f;let x=42;check(*g(&x)==42);}
