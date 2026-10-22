fn check(b:bool){if !b{let z=0;let _=1/z;}}type W<T>=T;type X<T>=W<T>;fn main(){check(X::<u32>::MAX==4294967295u32);}
