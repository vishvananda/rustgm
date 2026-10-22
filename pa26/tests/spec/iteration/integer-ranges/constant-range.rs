fn check(b:bool){if !b{let z=0;let _=1/z;}}const R:core::ops::Range<u8>=1u8..4u8;fn main(){let mut n=0u8;for x in R{n+=x;}check(n==6);}
