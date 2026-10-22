fn check(b:bool){if !b{let z=0;let _=1/z;}}const fn direct(r:&[u8])->u8{match r{[a,b]=>*a+*b,_=>0}}const N:u8=direct(&[1u8,2]);fn main(){check(N==3);}
