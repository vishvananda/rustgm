fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a>(p:&'a mut [u8;2])->&'a [u8]{p as &'a [u8]}fn main(){let mut a=[3,5];let s=cast(&mut a);check(s[1]==5);check(match s{[_,_]=>true,_=>false});}
