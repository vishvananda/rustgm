fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->u8{let [a,rest @ ..]=s else{return 0;};let [_,b]=rest else{return *a;};*a+*b}fn main(){check(f(&[])==0 && f(&[3])==3 && f(&[3,7,9])==12);}
