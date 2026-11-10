struct Tail<T:?Sized>{tag:u8,tail:T}
struct Other<T:?Sized>{tag:u8,tail:T}fn cast(x:&Tail<[u8;2]>)->&Other<[u8]>{x as &Other<[u8]>}fn main(){}
