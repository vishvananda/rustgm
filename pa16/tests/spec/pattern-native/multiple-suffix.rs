fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&mut [u8]){if let [..,a,b]=s{*a+=1;*b+=2;}}fn main(){let mut a=[1,2,3];f(&mut a);check(a[0]==1 && a[1]==3 && a[2]==5);}
