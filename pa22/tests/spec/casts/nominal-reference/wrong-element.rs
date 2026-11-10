struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast(x:&Tail<[u16;2]>)->&Tail<[u8]>{x as &Tail<[u8]>}fn main(){}
