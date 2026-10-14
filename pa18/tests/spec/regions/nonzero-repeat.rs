fn f<T:'static>(x:T)->&'static[T;1]{&[x;1]}
fn main(){}
