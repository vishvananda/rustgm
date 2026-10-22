const fn tail(x:&[u8])->u8{match x{[_,r @ ..,_]=>r[0],_=>0}}const N:u8=tail(&[7]);fn main(){let _=N;}
