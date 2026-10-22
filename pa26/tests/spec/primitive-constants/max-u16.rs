fn check(b:bool){if !b{let z=0;let _=1/z;}}const M:u16=u16::MAX;fn main(){check(M==65535u16);check(u16::MAX==M);}
