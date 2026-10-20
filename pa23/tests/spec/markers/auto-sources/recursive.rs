struct A<'a>(&'a B<'a>);struct B<'a>(&'a A<'a>);fn need<T:Send>(){}fn f(){need::<A<'static>>();}
fn main(){}
