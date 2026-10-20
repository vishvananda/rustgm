trait A{}struct W;impl A for W where W:Unpin{}impl Unpin for W where W:A{}fn need<T:Unpin>(){}fn f(){need::<W>();}
fn main(){}
