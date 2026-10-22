fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let a=[1u8,2,3,4];let [_,ref middle @ ..,_]=a;let middle:&[u8]=middle;let mut n=0u8;for x in middle{n+=*x;}check(n==5);}
