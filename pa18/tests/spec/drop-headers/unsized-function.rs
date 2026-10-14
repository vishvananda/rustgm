struct S<T:?Sized>{v:fn(*const T)}impl<T:?Sized> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
