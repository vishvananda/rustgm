fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn apply<'a>(f:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{f(x)}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply(keep,&x)==7);}