struct Tail<T:?Sized>{v:T}const fn mutate(r:&mut Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}const N:u8=mutate(&Tail{v:[1u8,2]});fn main(){let _=N;}
