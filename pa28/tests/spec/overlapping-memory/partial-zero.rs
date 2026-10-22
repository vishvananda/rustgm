union U{n:u8,wide:u32}const N:u32={let a=U{n:7};let mut b=11u32;unsafe{core::ptr::copy(&raw const a.wide,&raw mut b,0);}b};fn main(){let _=N;}
