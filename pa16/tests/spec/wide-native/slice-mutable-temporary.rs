fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&mut [u8]){s[1]+=1}fn main(){let a:&mut [u8]=&mut [7,8];f(a);check(a[1]==9);}
