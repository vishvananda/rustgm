struct W<'a>(&'a u8);unsafe impl<'a:'static> Sync for W<'a>{}struct V;impl Unpin for V where for<'a> W<'a>:Sync{}fn need<T:Unpin>(){}fn f(){need::<V>();}
fn main(){}
