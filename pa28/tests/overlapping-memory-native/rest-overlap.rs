fn main(){let mut a=[2u8,3,5,7,11];{let[_,ref mut r @ ..]=a;unsafe{core::ptr::copy(&raw const r[0],&raw mut r[1],3);}}check(a[0]==2&&a[1]==3&&a[2]==3&&a[3]==5&&a[4]==7);}
