const fn tail(x:&[u8])->&[u8]{match x{[_,r @ ..]=>r,_=>x}}const R:&[u8]=tail(b"abc");const N:u8=R[0];fn main(){let _=N;}
