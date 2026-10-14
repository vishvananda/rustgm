struct S<T>where [T;0]:Copy{v:T}impl<T> Drop for S<T>where [T;0]:Copy{fn drop(&mut self){}}
fn main(){}
