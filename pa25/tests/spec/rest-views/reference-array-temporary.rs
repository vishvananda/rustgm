const fn tail<'a>(a:&'a[&'a u8])->&'a[&'a u8]{match a{[_,r @ ..]=>r,_=>a}}static A:u8=7;const R:&[&u8]=tail(&[&A,&A]);const N:u8=*R[0];fn main(){let _=N;}
