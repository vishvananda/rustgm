struct Tail<T:?Sized>{v:T}const fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}const N:u8=take(&Tail{v:[1u16,2]});fn main(){let _=N;}
