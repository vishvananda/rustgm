fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->u8{match s{[]=>0,[x]=>*x,[x,y]=>*x+*y,[x,..,y]=>*x+*y}}fn main(){check(f(&[])==0 && f(&[7])==7 && f(&[3,4])==7 && f(&[2,1,8])==10);}
