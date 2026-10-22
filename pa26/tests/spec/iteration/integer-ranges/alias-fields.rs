fn check(b:bool){if !b{let z=0;let _=1/z;}}type R=core::ops::Range<u8>;fn main(){let mut r=R{start:2,end:4};r.start=3;let mut n=0u8;for x in r{n+=x;}check(n==3);}
