fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(x:&[u8;3])->u8{let [a,tail@..]=x;*a+tail[0]+tail[1]}fn main(){check(f(&[1,2,3])==6);}
