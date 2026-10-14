struct S<T>(T);impl<T=u8> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
