fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&mut [u8]){match s{[a,mid @ ..,b] if *a==1 && *b==4=>{*a=2;mid[0]=9;*b=5;},_=>()}}fn main(){let mut a=[1,2,4];f(&mut a);check(a[0]==2 && a[1]==9 && a[2]==5);}
