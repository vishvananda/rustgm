fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let f:fn(&[u8],&[u8])->bool=<[u8] as PartialEq>::eq;check(f(&[1,2],&[1,2]));}
