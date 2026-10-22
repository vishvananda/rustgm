fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut n=0u8;for x in 2u8..{n+=x;if x==4{break;}}check(n==9);}
