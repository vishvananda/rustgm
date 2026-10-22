fn check(b:bool){if !b{let z=0;let _=1/z;}}const M:u32=u32::MAX;fn main(){check(M==4294967295u32);check(u32::MAX==M);}
