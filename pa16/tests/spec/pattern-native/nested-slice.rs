fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[&[u8]])->u8{match s{[[],..,[_,v]]=>*v,[[x,..],..]=>*x,_=>0}}fn main(){let a:&[u8]=&[];let b:&[u8]=&[7,9];check(f(&[a,b])==9 && f(&[b,a])==7 && f(&[a])==0);}
