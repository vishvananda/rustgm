static X:u8=7;static Y:u8=11;const N:u8={let a=[&X,&Y];let mut b=[&X,&X];unsafe{core::ptr::copy(&raw const a[0],&raw mut b[0],2);}*b[1]};fn main(){let _=N;}
