struct Tail<T:?Sized>{v:T}const fn take(r:&Tail<[u8]>)->u8{r.v[2]}const N:u8=take(&Tail{v:[1u8,2]});fn main(){let _=N;}
