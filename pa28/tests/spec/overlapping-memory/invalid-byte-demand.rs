union U{n:u8,flag:bool}const N:bool={let a=U{n:2};let mut b=U{n:0};unsafe{core::ptr::copy(&raw const a.flag,&raw mut b.flag,1);b.flag}};fn main(){let _=N;}
