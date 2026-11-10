struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast<'a>(x:&'a &'a Tail<[u8;2]>)->&'a &'a Tail<[u8]>{x as &'a &'a Tail<[u8]>}fn main(){}
