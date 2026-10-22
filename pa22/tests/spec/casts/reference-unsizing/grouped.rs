fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[3u8,5];let s=(&a) as &[u8];check(s[1]==5);check(match s{[_,_]=>true,_=>false});}
