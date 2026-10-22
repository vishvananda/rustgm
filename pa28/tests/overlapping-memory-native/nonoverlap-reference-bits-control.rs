union U<'a>{n:usize,r:&'a u32}fn main(){let a=U{n:7};let mut b=U{n:0};unsafe{core::ptr::copy_nonoverlapping(&raw const a.r,&raw mut b.r,1);check(b.n==7);}}
