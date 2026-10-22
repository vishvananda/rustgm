fn main(){let mut a=[2u8,3,5,7];unsafe{core::ptr::copy(&raw const a[0],&raw mut a[2],2);}check(a[0]==2);check(a[1]==3);check(a[2]==2);check(a[3]==3);}
