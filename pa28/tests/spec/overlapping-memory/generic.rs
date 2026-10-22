const unsafe fn f<T>(a:*const T,b:*mut T,n:usize){unsafe{core::ptr::copy(a,b,n);}}const N:u64={let a=[2u64,3,5];let mut b=[0u64;3];unsafe{f(&raw const a[0],&raw mut b[0],3);}b[2]};fn main(){let _=N;}
