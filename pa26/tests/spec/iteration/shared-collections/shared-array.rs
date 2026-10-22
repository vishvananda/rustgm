fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let a=[1u8,2,3];let mut n=0u8;for &x in &a{n+=x;}check(n==6&&a[0]==1);}
