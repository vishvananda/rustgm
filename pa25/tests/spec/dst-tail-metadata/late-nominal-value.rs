fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Tail<T:?Sized>{v:T}const fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}const N:u8=take(&{Tail{v:[1u8,2]}});fn main(){check(N==3);}
