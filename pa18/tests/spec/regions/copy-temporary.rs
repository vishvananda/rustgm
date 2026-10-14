fn f<T:Copy+'static>(x:T)->&'static T{&{x}}
fn main(){}
