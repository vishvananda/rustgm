fn check(b:bool){if !b{let z=0;let _=1/z;}}use core::ops::Range as R;fn main(){let mut n=0u8;for x in (R{start:1u8,end:4u8}){n+=x;}check(n==6);}
