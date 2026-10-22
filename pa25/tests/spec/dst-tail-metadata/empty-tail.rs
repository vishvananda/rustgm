fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Tail<T:?Sized>{v:T}const fn empty(r:&Tail<[u8]>)->u8{match &r.v{[]=>37,_=>0}}const N:u8=empty(&Tail{v:[]});fn main(){check(N==37);}
