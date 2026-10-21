fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn infer<F:for<'a>Fn(&'a u8)->&'a u8>(f:F)->F{f}fn main(){let marker=0u8;let f=infer(|x|{if marker==0{x}else{x}});let n=7u8;check(*f(&n)==7);}
