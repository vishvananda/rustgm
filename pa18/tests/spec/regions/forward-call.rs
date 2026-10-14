fn id<T>(x:&T)->&T{x}fn f<'a,T>(x:T)->&'a T{id(&x)}
fn main(){}
