fn main(){let a=[2u64,3,5];let mut b=[0u64;3];let f:unsafe fn(*const u64,*mut u64,usize)=core::ptr::copy_nonoverlapping::<u64>;unsafe{f(&raw const a[0],&raw mut b[0],3);}check(b[2]==5);}
