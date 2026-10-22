fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let r=core::ops::Range{start:1u8,end:3u8};let mut n=0u8;for x in r{n+=x;}check(n==3);}
