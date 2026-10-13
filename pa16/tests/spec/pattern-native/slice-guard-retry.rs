fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->u8{let mut calls=0;match s{[x,..]|[..,x] if {calls+=1;*x==9}=>calls,_=>calls+10}}fn main(){check(f(&[2,9])==2 && f(&[9,2])==1 && f(&[2,3])==12 && f(&[])==10);}
