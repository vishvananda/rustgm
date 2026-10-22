fn main(){let a=[37u8,41];let mut b=[2u8,3,5,7];{let[_,ref mut r @ ..,_]=b;unsafe{core::ptr::copy(&raw const a[0],&raw mut r[0],2);}}check(b[0]==2);check(b[1]==37);check(b[2]==41);check(b[3]==7);}
