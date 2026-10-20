struct A<'a>(&'a B<'a>);struct B<'a>(&'a A<'a>,*const u8);fn need<T:Send>(){}fn f(){need::<A<'static>>();}
fn main(){}
