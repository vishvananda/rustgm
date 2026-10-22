const fn f()->u8{let mut a=[2u8,3,5,7];{let[_,ref mut r @ ..,_]=a;*r=[37,41];}a[1]+a[2]}const N:u8=f();fn main(){check(N==78);check(f()==78);}
