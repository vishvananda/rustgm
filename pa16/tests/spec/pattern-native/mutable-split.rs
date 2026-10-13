fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&mut [u8]){if let [a,mid @ ..,b]=s{*a+=1;mid[0]+=2;*b+=3;}}fn main(){let mut a=[1,2,3,4];f(&mut a);check(a[0]==2 && a[1]==4 && a[2]==3 && a[3]==7);}
