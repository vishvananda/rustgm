struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast<'a>(x:&'a Tail<[&'a [u8;2];1]>)->&'a Tail<[&'a [u8]]>{x as &'a Tail<[&'a [u8]]>}fn main(){}
