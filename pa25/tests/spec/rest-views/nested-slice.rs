const fn trim(x:&[u8])->&[u8]{match x{[_,r @ ..,_]=>match r{[_,s @ ..]=>s,_=>r},_=>x}}const R:&[u8]=trim(&[2,3,5,7,11]);const N:u8=R[0];fn main(){let _=N;}
