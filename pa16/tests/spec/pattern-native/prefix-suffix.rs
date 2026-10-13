fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->u8{match s{[1,..,3]=>7,[1,..]=>8,[..,3]=>9,_=>0}}fn main(){check(f(&[1,3])==7 && f(&[1])==8 && f(&[3])==9 && f(&[])==0 && f(&[0,1,3])==9);}
