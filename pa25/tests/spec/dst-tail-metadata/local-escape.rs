struct Tail<T:?Sized>{v:T}const fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}fn leak<'a>()->&'a Tail<[u8]>{let value=Tail{v:[1u8,2]};&value}fn main(){}
