fn main(){let x=7u8;let y=11u8;let a=[&x,&y];let mut b=[&x,&x];unsafe{core::ptr::copy_nonoverlapping(&raw const a[0],&raw mut b[0],2);}check(*b[1]==11);}
