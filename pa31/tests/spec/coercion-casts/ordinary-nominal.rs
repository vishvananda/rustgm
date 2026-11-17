struct Plain<T:?Sized>{pointer:*const T}
fn erase(b:Plain<[u8;2]>)->Plain<[u8]>{b as Plain<[u8]>}
fn main(){}
