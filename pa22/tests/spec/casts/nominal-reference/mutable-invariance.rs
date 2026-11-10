struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast<'a,'b>(x:&'b mut Tail<[&'a u8;1]>)->&'b mut Tail<[&'b u8]>where 'a:'b{x as &'b mut Tail<[&'b u8]>}fn main(){}
