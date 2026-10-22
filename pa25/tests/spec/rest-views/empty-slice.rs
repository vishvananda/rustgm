const fn tail(x:&[u8])->&[u8]{match x{[_,r @ ..]=>r,_=>x}}const R:&[u8]=tail(&[7]);const N:usize=R.len();fn main(){let _=N;}
