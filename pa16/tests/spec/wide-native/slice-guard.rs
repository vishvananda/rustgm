fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->u8{match s{t if t[0]==7=>t[1],_=>0}}fn main(){check(f(&[7,9])==9 && f(&[8,9])==0);}
