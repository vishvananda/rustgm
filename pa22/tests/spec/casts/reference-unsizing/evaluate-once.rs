fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make(n:&mut u8)->&'static [u8;2]{*n+=1;&[3,5]}fn main(){let mut n=0;let s=make(&mut n) as &[u8];check(n==1);check(s[1]==5);}
