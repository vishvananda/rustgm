fn check(b:bool){if !b{let z=0;let _=1/z;}}const M:usize=usize::MAX;fn main(){check(M==18446744073709551615usize);check(usize::MAX==M);}
