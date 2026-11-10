struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast(x:&Tail<[u8;2]>)->&mut Tail<[u8]>{x as &mut Tail<[u8]>}fn main(){}
