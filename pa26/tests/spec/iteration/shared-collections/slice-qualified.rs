fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let a=[1u8,2];let i=<[u8]>::iter(&a);let mut n=0u8;for x in i{n+=*x;}check(n==3);}
