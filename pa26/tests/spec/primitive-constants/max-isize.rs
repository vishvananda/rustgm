fn check(b:bool){if !b{let z=0;let _=1/z;}}const M:isize=isize::MAX;fn main(){check(M==9223372036854775807isize);check(isize::MAX==M);}
