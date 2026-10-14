type A<T> where B<T>:Copy =T;type B<U> where A<U>:Clone =U;fn f(x:A<u8>)->B<u8>{x}
fn main(){}
