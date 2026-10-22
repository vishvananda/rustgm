fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let mut n=0u8;for x in 0u8..10u8{if x==2{continue;}if x==5{break;}n+=x;}check(n==8);}
