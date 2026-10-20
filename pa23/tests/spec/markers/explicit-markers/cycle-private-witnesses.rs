struct A<'a>(&'a u8);impl<'a> Unpin for A<'a> where A<'static>:Unpin{}struct C(A<'static>,A<'static>);fn need<T:Unpin>(){}fn f(){need::<C>();}
fn main(){}
