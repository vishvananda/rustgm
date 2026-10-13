fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn get<'a>(s:&'a [u8])->&'a u8{&s[1]}fn main(){let a=[7,9];check(*get(&a)==9);}
