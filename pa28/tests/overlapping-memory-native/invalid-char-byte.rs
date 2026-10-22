union U{n:u32,c:char}fn main(){let a=U{n:0x110000};let mut b=U{n:0};unsafe{core::ptr::copy(&raw const a.c,&raw mut b.c,1);check(b.n==0x110000);}}
