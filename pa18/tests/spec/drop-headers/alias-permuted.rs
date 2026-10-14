struct S<T,U>(T,U);type A<T,U>=S<U,T>;impl<T,U> Drop for A<T,U>{fn drop(&mut self){}}
fn main(){}
