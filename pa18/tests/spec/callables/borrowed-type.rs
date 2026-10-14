fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T>(x:T)->T{x}fn f<'a>(x:&'a u8)->&'a u8{id(x)}fn main(){let n=11;check(*f(&n)==11);}
