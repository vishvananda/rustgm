fn check(b:bool){if !b{let z=0;let _=1/z;}}const M:u64=u64::MAX;fn main(){check(M==18446744073709551615u64);check(u64::MAX==M);}
