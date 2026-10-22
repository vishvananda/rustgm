const fn f()->u8{let a=[2u8,3,5,7];let[_,mut r @ ..,_]=a;r[0]=37;r[1]=41;r[0]+a[1]}const N:u8=f();fn main(){check(N==40);check(f()==40);}
