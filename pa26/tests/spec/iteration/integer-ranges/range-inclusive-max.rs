fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut n=0u8;for x in 255u8..=255u8{check(x==255);n+=1u8;}check(n==1);}
