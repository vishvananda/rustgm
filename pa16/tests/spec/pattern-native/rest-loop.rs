fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(mut s:&[u8])->u64{let mut n=0;while let [x,rest @ ..]=s{n+=*x as u64;s=rest;}n}fn main(){check(f(&[3,7,9])==19 && f(&[])==0);}
