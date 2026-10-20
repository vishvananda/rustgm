fn need<T:Unpin>(){}fn f<T:?Sized>(){need::<&mut T>();}
fn main(){}
