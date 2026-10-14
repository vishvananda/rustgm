struct S<T>where [T;0]:Copy{v:T}impl<T:Copy> Drop for S<T>{fn drop(&mut self){}}
fn main(){}
