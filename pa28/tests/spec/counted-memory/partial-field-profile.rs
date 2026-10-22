union U{n:u8,wide:u32}const N:u8={let a=U{n:7};let mut b=U{n:0};unsafe{core::ptr::copy_nonoverlapping(&raw const a.wide,&raw mut b.wide,1);b.n}};fn main(){let _=N;}
