struct Tail<P,T:?Sized>{prefix:P,tail:T}fn cast(x:&Tail<u8,[u8;2]>)->&Tail<u16,[u8]>{x as &Tail<u16,[u8]>}fn main(){}
