union U{n:u8,wide:u32}const N:u8={let a=U{n:7};let mut b=U{wide:0};unsafe{core::ptr::copy(&raw const a,&raw mut b,1);b.n}};fn main(){let _=N;}
