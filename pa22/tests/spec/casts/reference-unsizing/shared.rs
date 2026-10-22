fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a>(p:&'a [u8;3])->&'a [u8]{p as &'a [u8]}fn main(){let a=[3,5,7];let s=cast(&a);check(s[1]==5);check(match s{[_,_,_]=>true,_=>false});}
