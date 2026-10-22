union U{n:u8,b:bool}fn main(){let a=U{n:2};let mut b=U{n:0};unsafe{core::ptr::copy(&raw const a.b,&raw mut b.b,1);check(b.n==2);}}
