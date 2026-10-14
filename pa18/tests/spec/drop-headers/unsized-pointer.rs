struct S<T:?Sized>{v:*const T}impl<T:?Sized> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
