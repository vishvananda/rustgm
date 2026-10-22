const fn peel(mut a:&[u8],n:usize)->&[u8]{let mut i=0;while i<n{a=match a{[_,r @ ..]=>r,_=>a};i=i+1;}a}const R:&[u8]=peel(&[7;1_000_000],128);const N:u8=R[0];fn main(){let _=N;}
