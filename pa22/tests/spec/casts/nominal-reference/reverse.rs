struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast(x:&Tail<[u8]>)->&Tail<[u8;2]>{x as &Tail<[u8;2]>}fn main(){}
