fn check(b:bool){if !b{let z=0;let _=1/z;}}mod m{pub type A=usize;}type W=m::A;fn main(){check(W::MAX==18446744073709551615usize);}
