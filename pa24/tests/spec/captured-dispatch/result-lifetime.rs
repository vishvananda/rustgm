fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn use_it<'a>(x:&'a u8)->&'a u8{let f=move||x;let p:&dyn Fn()->&'a u8=&f;p()}fn main(){let x=7u8;check(*use_it(&x)==7);}
