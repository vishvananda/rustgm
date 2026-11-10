struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast<'a>(x:&'a Tail<[u8;2]>)->&'static Tail<[u8]>{x as &'static Tail<[u8]>}fn main(){}
