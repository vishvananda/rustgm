fn check(b:bool){if !b{let z=0;let _=1/z;}}const N:u8=take(&{Tail{v:[1u8,2]}});const fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}struct Tail<T:?Sized>{v:T}fn main(){check(N==3);}
