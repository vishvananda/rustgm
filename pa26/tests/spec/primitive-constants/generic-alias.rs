fn check(b:bool){if !b{let z=0;let _=1/z;}}type W<T>=T;fn main(){check(W::<u16>::MAX==65535u16);}
