fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->u8{match s{[_,rest @ ..]|[rest @ ..,_] if rest[0]==7=>rest[1],_=>0}}fn main(){check(f(&[1,7,9])==9 && f(&[7,9,1])==9);}
