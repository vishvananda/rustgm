fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let a=[2u8,3,4];let mut n=0u8;for x in 0..a.len(){n+=a[x];}check(n==9);}
