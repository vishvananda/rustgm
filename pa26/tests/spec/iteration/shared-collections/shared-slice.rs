fn check(b:bool){if !b{let z=0;let _=1/z;}}fn sum(a:&[u8])->u8{let mut n=0u8;for x in a{n+=*x;}n}fn main(){check(sum(&[1u8,2,3])==6);}
