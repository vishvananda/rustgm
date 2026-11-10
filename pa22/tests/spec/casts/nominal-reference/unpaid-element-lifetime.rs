struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast<'a>(x:&'a Tail<[&'a u8;1]>)->&'a Tail<[&'static u8]>{x as &'a Tail<[&'static u8]>}fn main(){}
