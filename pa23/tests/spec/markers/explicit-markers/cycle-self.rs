struct W;impl Unpin for W where W:Unpin{}fn need<T:Unpin>(){}fn f(){need::<W>();}
fn main(){}
